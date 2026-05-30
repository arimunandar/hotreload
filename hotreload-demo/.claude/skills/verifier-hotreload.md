# Verify Hot Reload Injection

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
