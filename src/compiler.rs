use std::path::Path;

use crate::error::HotReloadError;

#[derive(Debug)]
#[allow(dead_code)]
pub struct CompilationResult {
    pub dylib_path: std::path::PathBuf,
    pub dylib_name: String,
}

/// Incremental compilation: only codegen the changed files, parse all for type-checking.
/// Falls back to full compilation on failure.
pub fn compile_incremental(
    changed_files: &[std::path::PathBuf],
    all_swift_files: &[std::path::PathBuf],
    sdk_path: &str,
    target: &str,
    output_dir: &Path,
    module_name: &str,
    module_search_paths: &[String],
) -> anyhow::Result<CompilationResult> {
    std::fs::create_dir_all(output_dir)?;

    let id = chrono_id();
    let dylib_name = format!("injection_{}.dylib", id);
    let dylib_path = output_dir.join(&dylib_name);

    // Canonicalize paths so absolute/relative comparisons work
    let canonical_changed: Vec<std::path::PathBuf> = changed_files
        .iter()
        .filter_map(|p| std::fs::canonicalize(p).ok())
        .collect();

    // Create sanitized copies of context files that use unsupported macros
    // (e.g. @Generable) — strip the macro annotations so type definitions
    // are still available for type-checking without macro plugin errors
    let temp_dir = output_dir.join("_context");
    let _ = std::fs::create_dir_all(&temp_dir);
    let mut context_files: Vec<std::path::PathBuf> = Vec::new();
    let mut sanitized_count = 0;

    for f in all_swift_files {
        let content = std::fs::read_to_string(f).unwrap_or_default();
        if content.contains("@Generable") || content.contains("import FoundationModels") {
            let sanitized = content
                .lines()
                .map(|line| {
                    let trimmed = line.trim();
                    if trimmed.starts_with("@Generable") || trimmed.starts_with("import FoundationModels") {
                        format!("// {}", line)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            let name = f.file_name().unwrap_or_default();
            let dest = temp_dir.join(name);
            let _ = std::fs::write(&dest, sanitized);
            context_files.push(dest);
            sanitized_count += 1;
        } else {
            context_files.push(f.clone());
        }
    }

    if sanitized_count > 0 {
        tracing::info!("Sanitized {} file(s) with unsupported macros", sanitized_count);
    }

    tracing::info!(
        "Incremental: compiling {} file(s), {} context",
        canonical_changed.len(),
        context_files.len()
    );

    // Step 1: Compile each changed file to its own .o (in parallel)
    let obj_paths: Vec<_> = canonical_changed.iter().enumerate()
        .map(|(i, _)| output_dir.join(format!("injection_{}_{}.o", id, i)))
        .collect();

    let mut children: Vec<_> = canonical_changed.iter().enumerate()
        .map(|(i, primary)| {
            let mut cmd = std::process::Command::new("xcrun");
            cmd.arg("swift-frontend").arg("-c");
            cmd.arg("-primary-file").arg(primary);

            // Pass context files (excluding primary)
            for f in &context_files {
                let f_canonical = std::fs::canonicalize(f).unwrap_or_else(|_| f.clone());
                if f_canonical != *primary {
                    cmd.arg(f);
                }
            }

            cmd.args([
                "-module-name", module_name,
                "-target", target,
                "-sdk", sdk_path,
                "-enable-implicit-dynamic",
                "-enable-testing",
                "-Onone",
                "-parse-as-library",
            ]);

            // Load Xcode's macro plugins so @Observable etc. work
            let plugin_dir = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/host/plugins";
            if std::path::Path::new(plugin_dir).exists() {
                cmd.arg("-plugin-path").arg(plugin_dir);
            }

            // Add module search paths so the compiler can find pre-built
            // .swiftmodule files (e.g. HotReloadKit from DerivedData)
            for flag in module_search_paths {
                cmd.arg(flag);
            }

            cmd.arg("-o").arg(&obj_paths[i]);

            cmd.spawn()
                .map_err(|e| HotReloadError::CompilationFailed(format!("swift-frontend spawn failed: {}", e)))
        })
        .collect::<Result<Vec<_>, _>>()?;

    for child in &mut children {
        let status = child.wait()
            .map_err(|e| HotReloadError::CompilationFailed(format!("swift-frontend failed: {}", e)))?;

        if !status.success() {
            for o in &obj_paths { let _ = std::fs::remove_file(o); }
            tracing::warn!("Incremental compilation failed, falling back to full");
            return compile_all(all_swift_files, sdk_path, target, output_dir, module_name, module_search_paths);
        }
    }

    // Step 2: Link all .o files → .dylib
    let mut link = std::process::Command::new("xcrun");
    link.args([
        "clang",
        "-target", target,
        "-isysroot", sdk_path,
        "-shared",
        "-Xlinker", "-interposable",
        "-Xlinker", "-undefined",
        "-Xlinker", "dynamic_lookup",
    ]);
    for o in &obj_paths { link.arg(o); }
    link.arg("-L").arg(format!("{}/usr/lib/swift", sdk_path))
        .arg("-L").arg(swift_lib_dir(target))
        .args([
            "-framework", "SwiftUI",
            "-framework", "Foundation",
            "-lswiftCore",
            "-o",
        ]).arg(&dylib_path);

    let link_status = link.status()
        .map_err(|e| HotReloadError::CompilationFailed(format!("linker failed: {}", e)))?;

    for o in &obj_paths { let _ = std::fs::remove_file(o); }

    if !link_status.success() {
        tracing::warn!("Incremental link failed, falling back to full");
        return compile_all(all_swift_files, sdk_path, target, output_dir, module_name, module_search_paths);
    }

    let metadata = std::fs::metadata(&dylib_path)
        .map_err(|e| HotReloadError::CompilationFailed(format!("Dylib not created: {}", e)))?;

    tracing::info!(
        "Incremental compilation successful: {} ({} bytes)",
        dylib_path.display(),
        metadata.len()
    );

    Ok(CompilationResult { dylib_path, dylib_name })
}

/// Full compilation: compile all files together into a dylib (fallback).
pub fn compile_all(
    swift_files: &[std::path::PathBuf],
    sdk_path: &str,
    target: &str,
    output_dir: &Path,
    module_name: &str,
    module_search_paths: &[String],
) -> anyhow::Result<CompilationResult> {
    std::fs::create_dir_all(output_dir)?;

    let dylib_name = format!("injection_{}.dylib", chrono_id());
    let dylib_path = output_dir.join(&dylib_name);

    tracing::info!(
        "Full: compiling {} files -> {}",
        swift_files.len(),
        dylib_path.display()
    );

    let mut cmd = std::process::Command::new("xcrun");
    cmd.args([
        "swiftc",
        "-target", target,
        "-sdk", sdk_path,
        "-emit-library",
        "-module-name", module_name,
        "-Xlinker", "-interposable",
        "-Xfrontend", "-enable-implicit-dynamic",
        "-Xfrontend", "-enable-testing",
        "-Onone",
        "-parse-as-library",
        "-g",
        "-framework", "SwiftUI",
        "-framework", "Foundation",
        "-o",
    ]).arg(&dylib_path);

    // Add module search paths so the compiler can find pre-built
    // .swiftmodule files (e.g. HotReloadKit from DerivedData)
    for msp in module_search_paths {
        cmd.arg("-I").arg(msp);
    }

    for file in swift_files {
        cmd.arg(file);
    }

    let status = cmd.status()
        .map_err(|e| HotReloadError::CompilationFailed(format!("Failed to run swiftc: {}", e)))?;

    if !status.success() {
        return Err(HotReloadError::CompilationFailed(format!(
            "swiftc exited with code {:?}", status.code()
        )).into());
    }

    let metadata = std::fs::metadata(&dylib_path)
        .map_err(|e| HotReloadError::CompilationFailed(format!("Dylib not created: {}", e)))?;

    tracing::info!(
        "Compilation successful: {} ({} bytes)",
        dylib_path.display(),
        metadata.len()
    );

    Ok(CompilationResult { dylib_path, dylib_name })
}

pub fn find_swift_files(dirs: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for dir in dirs {
        if !dir.exists() {
            tracing::warn!("Source directory does not exist: {}", dir.display());
            continue;
        }
        collect_swift_files(dir, &mut files);
    }
    files
}

fn collect_swift_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.is_symlink() {
                collect_swift_files(&path, files);
            } else if path.extension().map_or(false, |e| e == "swift") {
                files.push(path);
            }
        }
    }
}

// TODO: The HotReloadKit InjectionLoader.swift currently has a hardcoded `excludedTypes`
// list (e.g. "TodoStore"). This should be auto-detected at compile time — either by
// parsing class definitions in the source files and filtering out ObservableObject
// subclasses, or by reading a config entry. See HotReloadKit repo for details.

/// Discover module search paths from Xcode DerivedData so the compiler can resolve
/// imports (e.g. `import HotReloadKit`) without needing the package source files.
///
/// Returns paths to directories containing `.swiftmodule` files for the given scheme.
/// If DerivedData is not found or the scheme has not been built, returns an empty vec
/// (compilation still works for projects that don't depend on external SPM modules).
pub fn find_module_search_paths(project_root: &Path, scheme: &str) -> Vec<String> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    // Search both standard Xcode DerivedData AND XcodeBuildMCP workspaces
    let mut search_roots = vec![
        std::path::PathBuf::from(&home).join("Library/Developer/Xcode/DerivedData"),
    ];
    let xcodebuildmcp = std::path::PathBuf::from(&home).join("Library/Developer/XcodeBuildMCP/workspaces");
    if xcodebuildmcp.exists() {
        if let Ok(entries) = std::fs::read_dir(&xcodebuildmcp) {
            for entry in entries.flatten() {
                let dd = entry.path().join("DerivedData");
                if dd.exists() {
                    search_roots.push(dd);
                }
            }
        }
    }

    // Collect ALL matching DerivedData directories, prefer the most recently modified
    let mut all_scheme_dirs: Vec<std::path::PathBuf> = Vec::new();
    for root in &search_roots {
        if !root.exists() { continue; }
        if let Some(d) = find_derived_data_dir(root, scheme, project_root) {
            all_scheme_dirs.push(d);
        }
    }
    // Sort: prefer dirs with SourcePackages, then by modification time (newest first)
    all_scheme_dirs.sort_by(|a, b| {
        let a_has_src = a.join("SourcePackages").exists();
        let b_has_src = b.join("SourcePackages").exists();
        if a_has_src != b_has_src {
            return b_has_src.cmp(&a_has_src);
        }
        let ma = std::fs::metadata(a).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let mb = std::fs::metadata(b).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        mb.cmp(&ma)
    });
    let scheme_dir = match all_scheme_dirs.first() {
        Some(d) => d.clone(),
        None => {
            tracing::debug!("No DerivedData directory found for scheme '{}'", scheme);
            return Vec::new();
        }
    };
    tracing::info!("Using DerivedData: {}", scheme_dir.display());

    let mut paths = Vec::new();

    // 1. Build/Products/Debug-iphonesimulator/ — Swift modules + frameworks
    let products_dir = scheme_dir.join("Build/Products/Debug-iphonesimulator");
    if products_dir.exists() {
        let p = products_dir.to_string_lossy().to_string();
        paths.push(format!("-I{}", p));
        paths.push(format!("-F{}", p));
        // PackageFrameworks subdirectory for SPM framework products
        let pkg_fw = products_dir.join("PackageFrameworks");
        if pkg_fw.exists() {
            paths.push(format!("-F{}", pkg_fw.to_string_lossy()));
        }
        tracing::info!("Module search path (products): {}", products_dir.display());
    }

    // 2. Build/Intermediates.noindex/ — per-target .swiftmodule and .modulemap files
    let intermediates = scheme_dir.join("Build/Intermediates.noindex");
    if intermediates.exists() {
        if let Ok(entries) = std::fs::read_dir(&intermediates) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.extension().map_or(false, |e| e == "build") {
                    let debug_sim = p.join("Debug-iphonesimulator");
                    if !debug_sim.exists() { continue; }
                    if let Ok(sub_entries) = std::fs::read_dir(&debug_sim) {
                        for sub in sub_entries.flatten() {
                            let sub_path = sub.path();
                            if !sub_path.is_dir() { continue; }
                            if sub_path.extension().map_or(false, |e| e == "build") {
                                // Swift module objects
                                let objects = sub_path.join("Objects-normal/arm64");
                                if objects.exists() {
                                    paths.push(format!("-I{}", objects.to_string_lossy()));
                                    tracing::info!("Module search path (intermediates): {}", objects.display());
                                }
                                // C/ObjC modulemaps — need explicit -fmodule-map-file for swift-frontend
                                if let Ok(files) = std::fs::read_dir(&sub_path) {
                                    for f in files.flatten() {
                                        let fp = f.path();
                                        if fp.extension().map_or(false, |ext| ext == "modulemap") {
                                            paths.push(format!("-Xcc"));
                                            paths.push(format!("-fmodule-map-file={}", fp.to_string_lossy()));
                                            tracing::info!("Module map: {}", fp.display());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. SourcePackages/checkouts/ — SPM C module modulemaps (e.g. GRDBSQLite)
    let source_packages = scheme_dir.join("SourcePackages/checkouts");
    if source_packages.exists() {
        fn find_modulemaps(dir: &Path, paths: &mut Vec<String>, depth: u8) {
            if depth > 5 { return; }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() && !p.is_symlink() {
                        find_modulemaps(&p, paths, depth + 1);
                    } else if p.file_name().map_or(false, |n| n == "module.modulemap") {
                        paths.push("-Xcc".to_string());
                        paths.push(format!("-fmodule-map-file={}", p.to_string_lossy()));
                        // Also add the parent as include path for headers
                        if let Some(parent) = p.parent() {
                            paths.push(format!("-I{}", parent.to_string_lossy()));
                        }
                        tracing::info!("Module map (SPM): {}", p.display());
                    }
                }
            }
        }
        find_modulemaps(&source_packages, &mut paths, 0);
    }

    paths
}

/// Find the DerivedData directory matching a given scheme.
/// Xcode creates directories named "<SchemeOrProject>-<hash>".
/// We first try matching by scheme name prefix, then validate using
/// the info.plist WorkspacePath if available.
fn find_derived_data_dir(
    derived_data: &Path,
    scheme: &str,
    project_root: &Path,
) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(derived_data).ok()?;

    let canonical_root = std::fs::canonicalize(project_root).ok();

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let name = dir.file_name()?.to_string_lossy().to_string();
        // Xcode uses "<Name>-<hash>" where Name matches the scheme/project
        if !name.starts_with(scheme) {
            continue;
        }
        // Verify the char after scheme name is '-' (to avoid "MyAppTests-xxx" matching "MyApp")
        let rest = &name[scheme.len()..];
        if !rest.starts_with('-') {
            continue;
        }
        candidates.push(dir);
    }

    // If we have a single candidate, use it directly
    if candidates.len() == 1 {
        return Some(candidates.remove(0));
    }

    // Multiple candidates: validate via info.plist's WorkspacePath
    if let Some(ref root) = canonical_root {
        for c in &candidates {
            let info_plist = c.join("info.plist");
            if let Ok(content) = std::fs::read_to_string(&info_plist) {
                if content.contains(&root.to_string_lossy().to_string()) {
                    return Some(c.clone());
                }
            }
        }
    }

    // Fall back to most recently modified
    candidates.sort_by(|a, b| {
        let ma = std::fs::metadata(a).and_then(|m| m.modified()).ok();
        let mb = std::fs::metadata(b).and_then(|m| m.modified()).ok();
        mb.cmp(&ma)
    });
    candidates.into_iter().next()
}

pub fn detect_target_for_deployment(scheme_target: &str, project_root: &Path, scheme: &str) -> String {
    // If the target already has a version, use it as-is
    let has_version = scheme_target.contains("ios1") || scheme_target.contains("ios2");
    if has_version {
        return scheme_target.to_string();
    }

    // Try to read the deployment target from Xcode build settings
    if let Ok(output) = std::process::Command::new("xcodebuild")
        .args(["-showBuildSettings", "-scheme", scheme])
        .current_dir(project_root)
        .output()
    {
        let settings = String::from_utf8_lossy(&output.stdout);
        for line in settings.lines() {
            if line.contains("IPHONEOS_DEPLOYMENT_TARGET") && !line.contains("RECOMMENDED") && !line.contains("SETTING_NAME") {
                if let Some(version) = line.split('=').nth(1) {
                    let v = version.trim();
                    if !v.is_empty() {
                        let target = scheme_target.replace("ios", &format!("ios{}", v));
                        tracing::info!("Deployment target: {}", target);
                        return target;
                    }
                }
            }
        }
    }

    // Fallback
    scheme_target.replace("ios", "ios17.0")
}

fn swift_lib_dir(target: &str) -> String {
    let platform = if target.contains("simulator") { "iphonesimulator" } else { "iphoneos" };
    format!(
        "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/{}",
        platform
    )
}

fn chrono_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos)
}
