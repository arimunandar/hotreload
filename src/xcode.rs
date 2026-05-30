use std::path::{Path, PathBuf};

use crate::error::HotReloadError;

/// Find .xcodeproj or .xcworkspace in the project root
pub fn detect_project(project_root: &Path) -> anyhow::Result<(PathBuf, Option<PathBuf>)> {
    let mut xcodeproj: Option<PathBuf> = None;
    let mut xcworkspace: Option<PathBuf> = None;

    for entry in std::fs::read_dir(project_root)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("xcodeproj") => xcodeproj = Some(path),
                Some("xcworkspace") if path.is_dir() => xcworkspace = Some(path),
                _ => {}
            }
        }
    }

    match (xcworkspace, xcodeproj) {
        (Some(ws), xcodeproj) => {
            // Prefer workspace over project
            let proj = find_project_in_workspace(&ws).or(xcodeproj).unwrap_or_else(|| {
                PathBuf::from("project")
            });
            Ok((proj, Some(ws)))
        }
        (None, Some(proj)) => Ok((proj, None)),
        (None, None) => Err(HotReloadError::ProjectNotFound(
            project_root.display().to_string(),
        )
        .into()),
    }
}

/// Try to find the .xcodeproj inside an .xcworkspace's contents.xcworkspacedata
fn find_project_in_workspace(workspace: &Path) -> Option<PathBuf> {
    let data_path = workspace.join("contents.xcworkspacedata");
    if !data_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(data_path).ok()?;
    // Parse workspace data for file references
    for line in content.lines() {
        if let Some(ref_start) = line.find("group:") {
            let rest = &line[ref_start + 6..];
            if let Some(end) = rest.find("\"") {
                let rel_path = &rest[..end];
                let candidate = workspace.parent()?.join(rel_path);
                if candidate.extension().map_or(false, |e| e == "xcodeproj") {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

/// List available schemes via xcodebuild -list
pub fn list_schemes(
    project_root: &Path,
    project: &Path,
    workspace: Option<&Path>,
) -> anyhow::Result<Vec<String>> {
    let mut cmd = std::process::Command::new("xcodebuild");
    cmd.current_dir(project_root);

    if let Some(ws) = workspace {
        cmd.arg("-workspace").arg(ws);
    } else {
        cmd.arg("-project").arg(project);
    }
    cmd.arg("-list");

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("xcodebuild -list stderr: {}", stderr);
        return Err(HotReloadError::Generic(format!(
            "xcodebuild -list failed: {}",
            stderr.trim()
        ))
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut schemes = Vec::new();
    let mut in_schemes = false;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Schemes:") {
            in_schemes = true;
            continue;
        }
        if in_schemes {
            if trimmed.is_empty() || trimmed.starts_with('(') {
                break;
            }
            if !trimmed.starts_with('-') && !trimmed.starts_with('*') {
                schemes.push(trimmed.to_string());
            }
        }
    }

    Ok(schemes)
}

/// Run xcodebuild -showBuildSettings and parse a specific setting
pub fn get_build_setting(
    project_root: &Path,
    scheme: &str,
    setting: &str,
) -> anyhow::Result<Option<String>> {
    let output = std::process::Command::new("xcodebuild")
        .current_dir(project_root)
        .args(["-scheme", scheme, "-showBuildSettings"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pattern = format!(" {} = ", setting);

    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(idx) = trimmed.find(&pattern) {
            let value = trimmed[idx + pattern.len()..].to_string();
            return Ok(Some(value));
        }
    }

    Ok(None)
}

/// Detect the Swift target triple for the iOS simulator
pub fn detect_target(project_root: &Path, scheme: &str) -> anyhow::Result<Option<String>> {
    // First try from build settings
    let arch = get_build_setting(project_root, scheme, "ARCHS")?;
    let sdk = get_build_setting(project_root, scheme, "SDK_NAME")?;

    let arch = arch.unwrap_or_else(|| {
        // Default to arm64 on Apple Silicon
        "arm64".to_string()
    });

    let target = match sdk.as_deref() {
        Some(s) if s.contains("iphonesimulator") => {
            format!("{}-apple-ios-simulator", arch)
        }
        Some(s) if s.contains("iphoneos") => {
            format!("{}-apple-ios", arch)
        }
        _ => {
            format!("{}-apple-ios-simulator", arch)
        }
    };

    Ok(Some(target))
}

/// Detect the SDK path
pub fn detect_sdk(sdk: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("xcrun")
        .args(["--sdk", sdk, "--show-sdk-path"])
        .output()?;

    if !output.status.success() {
        return Err(HotReloadError::Generic(format!(
            "Failed to detect SDK path: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
        .into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Check if the project has required build settings for injection
#[allow(dead_code)]
pub fn validate_build_settings(project_root: &Path, scheme: &str) -> anyhow::Result<Vec<String>> {
    let mut warnings = Vec::new();

    let ldflags = get_build_setting(project_root, scheme, "OTHER_LDFLAGS")?
        .unwrap_or_default();
    if !ldflags.contains("-interposable") {
        warnings.push(
            "OTHER_LDFLAGS should include '-Xlinker -interposable' for symbol interposition"
                .to_string(),
        );
    }

    let wmo = get_build_setting(project_root, scheme, "SWIFT_WHOLE_MODULE_OPTIMIZATION")?
        .unwrap_or_default();
    if wmo.eq_ignore_ascii_case("YES") {
        warnings.push(
            "SWIFT_WHOLE_MODULE_OPTIMIZATION should be set to NO".to_string(),
        );
    }

    Ok(warnings)
}
