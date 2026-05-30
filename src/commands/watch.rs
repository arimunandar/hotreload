use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::compiler;
use crate::config::Config;
use crate::injector;
use crate::server;
use crate::watcher;

pub fn run(
    project_root: &Path,
    http_port: u16,
    app_port: u16,
    app_host: &str,
) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        run_async(project_root, http_port, app_port, app_host).await
    })
}

async fn run_async(
    project_root: &Path,
    http_port: u16,
    app_port: u16,
    app_host: &str,
) -> anyhow::Result<()> {
    // Load config
    let config = Config::load(project_root)?;
    let dylib_dir = Config::config_dir(project_root).join("dylibs");
    std::fs::create_dir_all(&dylib_dir)?;

    let sdk_path = config
        .project
        .sdk_path
        .clone()
        .unwrap_or_else(|| crate::xcode::detect_sdk("iphonesimulator").unwrap_or_default());
    let target = config.project.target.clone().unwrap_or_else(|| {
        "arm64-apple-ios-simulator".to_string()
    });

    tracing::info!("SDK path: {}", sdk_path);
    tracing::info!("Target: {}", target);
    tracing::info!("Dylib dir: {}", dylib_dir.display());

    // Start HTTP file server
    let (actual_port, shutdown_tx) =
        server::start_file_server_with_port(dylib_dir.clone(), http_port).await?;

    // Start file watcher
    let (event_tx, mut event_rx) = mpsc::channel::<Vec<String>>(256);

    let raw_paths = if config.injection.watch_paths.is_empty() {
        vec!["Sources".to_string()]
    } else {
        config.injection.watch_paths.clone()
    };

    // Resolve watch paths relative to project root
    let watch_paths: Vec<String> = raw_paths
        .iter()
        .map(|p| {
            let path = std::path::Path::new(p);
            if path.is_relative() {
                project_root.join(p).to_string_lossy().to_string()
            } else {
                p.clone()
            }
        })
        .collect();

    let _watcher = watcher::start_watcher(
        &watch_paths,
        config.injection.debounce_ms,
        event_tx,
    )?;

    // Convert watch paths to PathBufs for file watching
    let watch_dirs: Vec<PathBuf> = watch_paths.iter().map(PathBuf::from).collect();

    // Discover module search paths from DerivedData so the compiler can find
    // pre-built Swift modules (e.g. HotReloadKit) without needing their source.
    let module_search_paths = compiler::find_module_search_paths(
        project_root,
        &config.project.scheme,
    );

    // Use versioned target for proper iOS availability
    let compile_target = compiler::detect_target_for_deployment(&target, project_root, &config.project.scheme);

    // Module name must match the app's module for symbol interposition
    let module_name = config
        .project
        .module_name
        .clone()
        .unwrap_or_else(|| config.project.scheme.clone());

    println!("📡 HotReload watcher active");
    println!("   HTTP server: http://127.0.0.1:{}", actual_port);
    println!("   App TCP: {}:{}", app_host, app_port);
    println!("   Watching: {:?}", watch_paths);
    println!();
    println!("   Modify a .swift file to trigger injection...");
    println!();

    let dylib_dir = Arc::new(dylib_dir);

    // Event loop
    while let Some(changed_files) = event_rx.recv().await {
        let all_files = compiler::find_swift_files(&watch_dirs);

        if all_files.is_empty() {
            tracing::warn!("No Swift files found in watch directories");
            continue;
        }

        let changed_paths: Vec<std::path::PathBuf> = changed_files
            .iter()
            .map(std::path::PathBuf::from)
            .filter(|p| {
                p.extension().map_or(false, |e| e == "swift")
                    && p.exists()
                    && p.file_name().map_or(false, |n| !n.to_string_lossy().starts_with('.'))
            })
            .collect();

        let compile_result = if !changed_paths.is_empty() {
            tracing::info!("Change detected in {} file(s)", changed_paths.len());
            compiler::compile_incremental(
                &changed_paths, &all_files, &sdk_path, &compile_target, &dylib_dir, &module_name, &module_search_paths,
            )
        } else {
            continue;
        };

        match compile_result {
            Ok(result) => {
                tracing::info!("Compiled: {}", result.dylib_name);

                let dylib_host = if app_host == "127.0.0.1" || app_host == "localhost" {
                    "127.0.0.1".to_string()
                } else {
                    crate::commands::detect_local_ip().unwrap_or_else(|| "127.0.0.1".to_string())
                };
                let dylib_url = format!(
                    "http://{}:{}/dylib/{}",
                    dylib_host, actual_port, result.dylib_name
                );

                // Send injection command to app
                match injector::send_injection(app_host, app_port, &result.dylib_name, &dylib_url) {
                    Ok(response) => {
                        tracing::info!("✅ Injected: {} -> {}", result.dylib_name, response);
                        println!("✅ Injected: {} -> {}", result.dylib_name, response);
                    }
                    Err(e) => {
                        tracing::warn!("Injection failed: {} (is the app running?)", e);
                        println!("⚠️  Injection failed: {} (is the app running?)", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Compilation failed: {}", e);
                println!("❌ Compilation failed: {}", e);
            }
        }
    }

    // Graceful shutdown
    let _ = shutdown_tx.send(());
    tracing::info!("Watcher stopped");

    Ok(())
}
