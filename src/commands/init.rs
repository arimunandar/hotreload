use std::path::Path;

use crate::config::Config;
use crate::xcode;

const SKILL_HOTRELOAD: &str = r#"# Hot Reload — edit, inject, verify

Trigger: user says "/hotreload", asks to make UI changes with hot reload, or wants to test hot reload injection.

## Prerequisites

- The iOS app must be running on the simulator with HotReloadKit integrated
- `.hotreload/config.toml` must exist (run `hotreload init` if missing)

## Workflow

1. **Check connection** — run `hotreload status` to confirm the app is reachable. If it fails, tell the user to launch the app in the simulator first.

2. **Start the watcher** (if not already running) — run `hotreload watch` in the background. The watcher auto-detects changes, recompiles, and injects. If the user has a specific project path, use `hotreload watch --path <project>`.

3. **Edit the Swift file** — make the requested UI change. Prefer editing view body structs, modifiers, colors, text, layout values. Save the file.

4. **Wait for injection** — the watcher picks up the change automatically (debounce ~150ms). Wait 3-5 seconds for compile + inject to complete.

5. **Check the log** — run `hotreload log --last 30s` and look for:
   - `dlopen` success — the dylib was loaded
   - `TX patch` count — how many function pointers were swizzled
   - Any errors (compile failures, missing symbols)

6. **Screenshot** — capture the simulator screen to confirm the visual change. Use the XcodeBuildMCP screenshot tool if available (`mcp__XcodeBuildMCP__screenshot`), otherwise run `xcrun simctl io booted screenshot /tmp/hotreload-verify.png` and read the image.

## What works with hot reload

- SwiftUI `var body` changes (layout, modifiers, colors, text, images)
- Computed properties on structs
- Free functions and static methods
- Extension methods on structs/enums

## What does NOT work (requires full rebuild)

- Changes to **class** definitions (ObservableObject, UIViewController, etc.) — TX symbol patching cannot update class vtables
- Adding/removing stored properties
- Changes to protocols or protocol conformances
- Adding new types
- Changes to `@main` or app entry point

## One-shot injection

If the watcher is not running, you can inject a single file directly:

```bash
hotreload inject Sources/Views/MyView.swift
```

## Troubleshooting

- **"Could not connect"** — app not running or HotReloadKit not integrated. Launch the app first.
- **Compile error in log** — check the log output, fix the Swift error, save again.
- **0 TX patches** — the changed function may not be interposable. Ensure `OTHER_SWIFT_FLAGS` includes `-Xfrontend -enable-implicit-dynamic` in the Xcode build settings.
- **Changes not visible** — the view may need a state change to re-render. Try toggling a state value or navigating away and back.
"#;

const SKILL_VERIFIER: &str = r#"# Verify Hot Reload Injection

Trigger: after making a hot-reload change, or when the user asks to verify/confirm an injection succeeded.

## Verification steps

1. **Check logs** — run `hotreload log --last 30s` and parse the output:
   - Look for `dlopen` — confirms the dylib was loaded into the app process
   - Look for `TX patch` — note the count of patched function pointers
   - Look for any error lines (compile failures, symbol resolution errors)

2. **Take a screenshot** — use the XcodeBuildMCP screenshot tool (`mcp__XcodeBuildMCP__screenshot`) if available, otherwise run:
   ```bash
   xcrun simctl io booted screenshot /tmp/hotreload-verify.png
   ```
   Then read the screenshot image to visually confirm the change.

3. **Report result**:

   **PASS** when ALL of the following are true:
   - `dlopen` succeeded (no dlopen error in logs)
   - TX patch count > 0
   - Screenshot shows the expected visual change

   **FAIL** when ANY of the following are true:
   - No recent log entries (watcher may not be running)
   - dlopen error (compile failure or missing symbols)
   - TX patch count is 0 (function not interposable)
   - Screenshot does not reflect the expected change

## Report format

```
Injection: PASS/FAIL
  dlopen:     OK / ERROR: <message>
  TX patches: <count>
  Visual:     confirmed / not confirmed — <reason>
```

## Common failure causes

- **dlopen error** — usually a Swift compile error. Check the full log for the compiler diagnostic and fix the source file.
- **0 TX patches** — the file likely contains only class methods. Class files (ObservableObject, UIViewController) require a full rebuild.
- **Visual mismatch** — SwiftUI may cache the view. A state toggle or navigation may be needed to force a re-render.
"#;

pub fn run(project_root: &Path, force: bool) -> anyhow::Result<()> {
    // Always install/update Claude Code skills
    install_skills(project_root)?;

    let config_path = Config::config_path(project_root);

    if config_path.exists() && !force {
        anyhow::bail!(
            "Config already exists at {}. Use --force to overwrite.",
            config_path.display()
        );
    }

    tracing::info!("Scanning for Xcode projects in {}...", project_root.display());

    // Detect Xcode project
    let (project_path, workspace_path) = xcode::detect_project(project_root)?;
    let project_ref = workspace_path.as_ref().unwrap_or(&project_path);
    tracing::info!("Found project: {}", project_ref.display());

    // Find schemes
    let schemes = xcode::list_schemes(project_root, &project_path, workspace_path.as_deref())?;
    if schemes.is_empty() {
        anyhow::bail!("No iOS schemes found in {}", project_ref.display());
    }
    let scheme = &schemes[0];
    tracing::info!("Using scheme: {}", scheme);

    // Try to get build settings
    let target = xcode::detect_target(project_root, scheme)?;
    let sdk_path = xcode::detect_sdk("iphonesimulator")?;

    let mut config = Config::default();
    config.project.project_path = project_path;
    config.project.scheme = scheme.clone();
    config.project.module_name = Some(scheme.clone());
    config.project.target = target;
    config.project.sdk_path = Some(sdk_path);

    config.save(project_root)?;

    tracing::info!(
        "Initialization complete! Config written to {}",
        config_path.display()
    );

    // Try to auto-configure Xcode build settings
    let build_settings_ok = configure_xcode_build_settings(&config.project.project_path);

    println!();
    println!("✅  HotReload initialized!");
    println!();
    if build_settings_ok {
        println!("  Build settings configured automatically ✓");
        println!();
        println!("Next steps:");
        println!("  1. Add HotReloadKit in Xcode:");
        println!("     File > Add Package Dependencies > https://github.com/arimunandar/HotReloadKit");
        println!("  2. Start watching:");
        println!("     hotreload watch");
    } else {
        println!("Next steps:");
        println!("  1. Add HotReloadKit in Xcode:");
        println!("     File > Add Package Dependencies > https://github.com/arimunandar/HotReloadKit");
        println!("  2. Add build settings in Xcode (Debug configuration):");
        println!("     OTHER_LDFLAGS = $(inherited) -Xlinker -interposable");
        println!("     OTHER_SWIFT_FLAGS = $(inherited) -Xfrontend -enable-implicit-dynamic");
        println!("  3. Start watching:");
        println!("     hotreload watch");
    }
    println!();

    Ok(())
}

/// Try to auto-configure the Xcode project with build settings needed for hot reload.
/// Returns true if settings were successfully configured (or already present),
/// false if the pbxproj could not be parsed and the user needs to do it manually.
fn configure_xcode_build_settings(project_path: &Path) -> bool {
    let pbxproj_path = project_path.join("project.pbxproj");
    if !pbxproj_path.exists() {
        tracing::debug!("No pbxproj at {}", pbxproj_path.display());
        return false;
    }

    let content = match std::fs::read_to_string(&pbxproj_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Could not read pbxproj: {}", e);
            return false;
        }
    };

    let has_interposable = content.contains("-interposable");
    let has_implicit_dynamic = content.contains("-enable-implicit-dynamic");

    if has_interposable && has_implicit_dynamic {
        println!("📋 Build settings already configured (interposable + implicit-dynamic)");
        return true;
    }

    // Attempt to patch the Debug XCBuildConfiguration sections.
    // The pbxproj format is old-style plist. We look for buildSettings blocks
    // inside sections that reference "Debug" and add/modify the flags.
    //
    // This is intentionally conservative: if the structure doesn't match
    // expectations, we bail out and tell the user to do it manually.
    match patch_pbxproj_build_settings(&content, has_interposable, has_implicit_dynamic) {
        Some(patched) => {
            // Write a backup first
            let backup_path = pbxproj_path.with_extension("pbxproj.hotreload-backup");
            if let Err(e) = std::fs::copy(&pbxproj_path, &backup_path) {
                tracing::warn!("Could not create backup: {}", e);
                println!("⚠️  Could not create pbxproj backup: {}", e);
                println!("   Skipping auto-configuration. Please add build settings manually.");
                return false;
            }

            match std::fs::write(&pbxproj_path, patched) {
                Ok(_) => {
                    println!("📋 Auto-configured Xcode build settings (Debug):");
                    if !has_interposable {
                        println!("   + OTHER_LDFLAGS: -Xlinker -interposable");
                    }
                    if !has_implicit_dynamic {
                        println!("   + OTHER_SWIFT_FLAGS: -Xfrontend -enable-implicit-dynamic");
                    }
                    println!("   Backup saved to {}", backup_path.display());
                    true
                }
                Err(e) => {
                    tracing::warn!("Could not write patched pbxproj: {}", e);
                    println!("⚠️  Could not write patched pbxproj: {}", e);
                    // Try to restore backup
                    let _ = std::fs::copy(&backup_path, &pbxproj_path);
                    false
                }
            }
        }
        None => {
            println!("⚠️  Could not auto-configure pbxproj (unexpected format).");
            println!("   Please add the following build settings manually in Xcode (Debug config):");
            if !has_interposable {
                println!("   OTHER_LDFLAGS = $(inherited) -Xlinker -interposable");
            }
            if !has_implicit_dynamic {
                println!("   OTHER_SWIFT_FLAGS = $(inherited) -Xfrontend -enable-implicit-dynamic");
            }
            false
        }
    }
}

/// Patch the pbxproj content to add hot-reload build settings to Debug configurations.
/// Returns Some(patched_content) on success, None if the format couldn't be parsed.
fn patch_pbxproj_build_settings(
    content: &str,
    has_interposable: bool,
    has_implicit_dynamic: bool,
) -> Option<String> {
    let mut result = String::with_capacity(content.len() + 512);
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut patched_any = false;

    while i < lines.len() {
        let line = lines[i];
        result.push_str(line);
        result.push('\n');

        // Look for XCBuildConfiguration sections with "Debug" name.
        // Pattern: we find `name = Debug;` and then look back/forward for buildSettings.
        // Typical structure:
        //   <id> /* Debug */ = {
        //       isa = XCBuildConfiguration;
        //       buildSettings = {
        //           ...
        //       };
        //       name = Debug;
        //   };
        if line.trim() == "buildSettings = {" {
            // Check if this block belongs to a Debug configuration.
            // Look ahead for `name = Debug;`
            let mut is_debug = false;
            let mut brace_depth = 1;
            let mut j = i + 1;
            while j < lines.len() && brace_depth > 0 {
                let check = lines[j].trim();
                if check.contains('{') {
                    brace_depth += check.matches('{').count() as i32;
                }
                if check.contains('}') {
                    brace_depth -= check.matches('}').count() as i32;
                }
                if brace_depth == 0 {
                    // We've exited the buildSettings block; check the next few lines for name = Debug
                    let mut k = j + 1;
                    while k < lines.len() && k <= j + 3 {
                        if lines[k].trim().starts_with("name = Debug") || lines[k].trim().starts_with("name = \"Debug\"") {
                            is_debug = true;
                            break;
                        }
                        if lines[k].trim() == "};" || lines[k].trim().starts_with("name =") {
                            break;
                        }
                        k += 1;
                    }
                }
                j += 1;
            }

            if is_debug {
                // Now emit the buildSettings content, inserting our flags.
                // We need to find where OTHER_LDFLAGS and OTHER_SWIFT_FLAGS are (or where to add them).
                let mut depth = 1;
                let mut settings_end = i + 1;
                while settings_end < lines.len() && depth > 0 {
                    let check = lines[settings_end].trim();
                    if check.contains('{') {
                        depth += check.matches('{').count() as i32;
                    }
                    if check.contains('}') {
                        depth -= check.matches('}').count() as i32;
                    }
                    if depth == 0 {
                        break;
                    }
                    settings_end += 1;
                }
                // settings_end now points to the closing `};` of buildSettings

                // Detect indentation from existing lines
                let indent = if i + 1 < settings_end {
                    let sample = lines[i + 1];
                    let trimmed = sample.trim_start();
                    &sample[..sample.len() - trimmed.len()]
                } else {
                    "\t\t\t\t"
                };

                let mut ldflags_found = false;
                let mut swiftflags_found = false;

                // Process each line inside buildSettings
                let mut k = i + 1;
                while k < settings_end {
                    let sline = lines[k];
                    let strimmed = sline.trim();

                    // Patch OTHER_LDFLAGS
                    if !has_interposable && strimmed.starts_with("OTHER_LDFLAGS") {
                        ldflags_found = true;
                        if strimmed.contains("(") && strimmed.contains(")") && strimmed.contains(';') {
                            // Single-line: OTHER_LDFLAGS = "$(inherited) ...";
                            // Add our flags
                            let patched_line = if strimmed.contains("-interposable") {
                                sline.to_string()
                            } else {
                                sline.replace("\";", " -Xlinker -interposable\";")
                                    .replace(");", " -Xlinker -interposable);")
                            };
                            result.push_str(&patched_line);
                            result.push('\n');
                        } else if strimmed.ends_with('(') {
                            // Multi-line array: OTHER_LDFLAGS = (
                            result.push_str(sline);
                            result.push('\n');
                            k += 1;
                            // Copy existing entries
                            while k < settings_end {
                                let arr_line = lines[k].trim();
                                if arr_line == ");" {
                                    // Insert before closing
                                    result.push_str(indent);
                                    result.push_str("\t\"-Xlinker\",\n");
                                    result.push_str(indent);
                                    result.push_str("\t\"-interposable\",\n");
                                    result.push_str(lines[k]);
                                    result.push('\n');
                                    break;
                                }
                                result.push_str(lines[k]);
                                result.push('\n');
                                k += 1;
                            }
                        } else {
                            result.push_str(sline);
                            result.push('\n');
                        }
                        k += 1;
                        continue;
                    }

                    // Patch OTHER_SWIFT_FLAGS
                    if !has_implicit_dynamic && strimmed.starts_with("OTHER_SWIFT_FLAGS") {
                        swiftflags_found = true;
                        if strimmed.contains(';') && !strimmed.ends_with('(') {
                            let patched_line = if strimmed.contains("-enable-implicit-dynamic") {
                                sline.to_string()
                            } else {
                                sline.replace("\";", " -Xfrontend -enable-implicit-dynamic\";")
                                    .replace(");", " -Xfrontend -enable-implicit-dynamic);")
                            };
                            result.push_str(&patched_line);
                            result.push('\n');
                        } else if strimmed.ends_with('(') {
                            result.push_str(sline);
                            result.push('\n');
                            k += 1;
                            while k < settings_end {
                                let arr_line = lines[k].trim();
                                if arr_line == ");" {
                                    result.push_str(indent);
                                    result.push_str("\t\"-Xfrontend\",\n");
                                    result.push_str(indent);
                                    result.push_str("\t\"-enable-implicit-dynamic\",\n");
                                    result.push_str(lines[k]);
                                    result.push('\n');
                                    break;
                                }
                                result.push_str(lines[k]);
                                result.push('\n');
                                k += 1;
                            }
                        } else {
                            result.push_str(sline);
                            result.push('\n');
                        }
                        k += 1;
                        continue;
                    }

                    result.push_str(sline);
                    result.push('\n');
                    k += 1;
                }

                // If flags were not found, add them before the closing `};`
                if !has_interposable && !ldflags_found {
                    result.push_str(indent);
                    result.push_str("OTHER_LDFLAGS = (\n");
                    result.push_str(indent);
                    result.push_str("\t\"$(inherited)\",\n");
                    result.push_str(indent);
                    result.push_str("\t\"-Xlinker\",\n");
                    result.push_str(indent);
                    result.push_str("\t\"-interposable\",\n");
                    result.push_str(indent);
                    result.push_str(");\n");
                }
                if !has_implicit_dynamic && !swiftflags_found {
                    result.push_str(indent);
                    result.push_str("OTHER_SWIFT_FLAGS = (\n");
                    result.push_str(indent);
                    result.push_str("\t\"$(inherited)\",\n");
                    result.push_str(indent);
                    result.push_str("\t\"-Xfrontend\",\n");
                    result.push_str(indent);
                    result.push_str("\t\"-enable-implicit-dynamic\",\n");
                    result.push_str(indent);
                    result.push_str(");\n");
                }

                // Emit the closing `};` of buildSettings
                result.push_str(lines[settings_end]);
                result.push('\n');

                // Skip ahead past the lines we already processed
                i = settings_end + 1;
                patched_any = true;
                continue;
            }
        }

        i += 1;
    }

    if patched_any { Some(result) } else { None }
}

fn install_skills(project_root: &Path) -> anyhow::Result<()> {
    let skills_dir = project_root.join(".claude").join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    let hotreload_path = skills_dir.join("hotreload.md");
    std::fs::write(&hotreload_path, SKILL_HOTRELOAD)?;
    tracing::info!("Skill installed: {}", hotreload_path.display());

    let verifier_path = skills_dir.join("verifier-hotreload.md");
    std::fs::write(&verifier_path, SKILL_VERIFIER)?;
    tracing::info!("Skill installed: {}", verifier_path.display());

    println!("📎 Claude Code skills installed to .claude/skills/");

    Ok(())
}
