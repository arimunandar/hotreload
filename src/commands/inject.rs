use std::path::{Path, PathBuf};

use crate::compiler;
use crate::config::Config;
use crate::injector;
use crate::server;

pub fn run(
    project_root: &Path,
    file: &str,
    http_port: u16,
    app_port: u16,
    app_host: &str,
) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        run_async(project_root, file, http_port, app_port, app_host).await
    })
}

async fn run_async(
    project_root: &Path,
    _file: &str,
    http_port: u16,
    app_port: u16,
    app_host: &str,
) -> anyhow::Result<()> {
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

    // Find all Swift files in the project (app files for codegen)
    let app_dirs: Vec<PathBuf> = config
        .injection
        .watch_paths
        .iter()
        .map(|p| {
            let path = Path::new(p);
            if path.is_relative() {
                project_root.join(p)
            } else {
                path.to_path_buf()
            }
        })
        .collect();

    // Discover module search paths from DerivedData so the compiler can find
    // pre-built Swift modules (e.g. HotReloadKit) without needing their source.
    let module_search_paths = compiler::find_module_search_paths(
        project_root,
        &config.project.scheme,
    );

    let all_files = compiler::find_swift_files(&app_dirs);
    if all_files.is_empty() {
        anyhow::bail!("No Swift files found in source directories");
    }

    // Only compile the specified file; all others serve as type-checking context
    let changed_files = vec![std::path::PathBuf::from(_file)];

    tracing::info!("Compiling 1 file ({} total with context)", all_files.len());

    // Start the HTTP server briefly to serve the dylib
    let (actual_port, shutdown_tx) =
        server::start_file_server_with_port(dylib_dir.clone(), http_port).await?;

    let compile_target = compiler::detect_target_for_deployment(&target, project_root, &config.project.scheme);

    let module_name = config
        .project
        .module_name
        .clone()
        .unwrap_or_else(|| config.project.scheme.clone());

    let result = compiler::compile_incremental(&changed_files, &all_files, &sdk_path, &compile_target, &dylib_dir, &module_name, &module_search_paths)?;
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

    // Inject
    match injector::send_injection(app_host, app_port, &result.dylib_name, &dylib_url) {
        Ok(response) => {
            tracing::info!("✅ Injected: {} -> {}", result.dylib_name, response);
            println!("✅ Injected: {}", result.dylib_name);
        }
        Err(e) => {
            tracing::warn!("Injection failed: {}", e);
            println!("⚠️  Injection failed: {}", e);
        }
    }

    // Keep HTTP server alive so the app can fetch the dylib
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let _ = shutdown_tx.send(());
    Ok(())
}
