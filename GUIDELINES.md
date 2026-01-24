# PROJECT GUIDELINES

## 1. UI TEXT CASING (CRITICAL)
**STRICTLY ENFORCED: NO CAPITAL LETTERS IN USER INTERFACE.**

- **ALL** user-facing strings must be lowercase.
- **ALL** buttons, titles, menus, prompts, and error messages.
- **NO EXCEPTIONS.**

### Correct
- `task manager`
- `restart`
- `cancel`
- `error: file not found`
- `confirm restart? (y/n)`

### Incorrect
- `Task Manager`
- `Restart`
- `Cancel`
- `Error: File not found`
- `Confirm Restart? (Y/N)`

## 2. COMMITMENT
- Before submitting any UI code, verified that **ZERO** capital letters exist in the output.
- If you see a capital letter in the UI, it is a bug. Fix it immediately.

## 3. BUILD WORKFLOW (CRITICAL)
**Apps like `desktop.wasm` are separate binaries that do NOT hot-reload with `trunk serve`.**

- `trunk serve` only hot-reloads the **kernel** (`src/`).
- Apps in `apps/` (e.g., `desktop.wasm`) require running `build.ps1` to rebuild.
- After modifying any app code, run `build.ps1` to rebuild the wasm binaries.
- The rebuilt binaries are embedded in the filesystem and require a page refresh to take effect.

### Workflow
1. Edit kernel code (`src/`) → `trunk serve` auto-reloads
2. Edit app code (`apps/`) → run `build.ps1` → refresh page
