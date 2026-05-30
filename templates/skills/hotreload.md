# Hot Reload — edit, inject, verify

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
