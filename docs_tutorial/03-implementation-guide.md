# Implementation Guide (Decoupled Backend)

This guide explains the current architecture after the panic-safety refactor.

## 1. Module Responsibilities

| File | Responsibility |
| --- | --- |
| `src/ui/mod.rs` | Builds overlay window, grid UI, async receiver loop |
| `src/ui/input.rs` | Keyboard navigation and action dispatch trigger |
| `src/backend/hyprctl.rs` | Event listener, safe JSON fetching, focus worker queue |
| `src/backend/screencopy.rs` | Wayland screencopy capture path |

## 2. Safe Data Fetching Pipeline

The backend uses native `hyprctl` commands:

1. `hyprctl clients -j`
2. `hyprctl activewindow -j`
3. `hyprctl monitors -j`

Each command is parsed with `serde_json::from_slice` into private typed structs:

- `HyprctlClient`
- `HyprctlActiveWindow`
- `HyprctlMonitor`

All failures are handled with `Option`/`Result` checks and `.ok()?`-style exits.  
No `unwrap()` and no `expect()` in this data path.

## 3. Why This Is Panic-Safe

The previous design depended on third-party wrappers that could panic internally.  
The new design:

- Executes subprocesses directly.
- Treats command errors as normal runtime conditions.
- Skips only the failed refresh cycle.
- Keeps the process and UI alive.

## 4. Focus Dispatch Isolated from UI Thread

When user presses Enter:

1. UI enqueues the target address to a bounded `sync_channel`.
2. A dedicated worker thread executes:
   `hyprctl dispatch focuswindow address:<address>`
3. UI closes immediately without waiting.

This guarantees keyboard responsiveness even if compositor IPC is slow.

## 5. Observability

Focus dispatch failures are logged to:

- `stderr`
- `/tmp/window-switcher-focus.log`

The log includes exit code, stdout, and stderr for debugging compositor-side errors.

## 6. Keyboard Input Stability

`EventControllerKey` runs in `PropagationPhase::Capture` and the window is explicitly focused.  
This prevents child widgets from swallowing Enter/Escape unexpectedly.

## 7. Garbage Collection Integration

`refresh_and_send` builds an `active_addresses` set from mapped clients and triggers async GC:

- keep `<address>.png` if address is still active
- delete stale files otherwise

GC is best-effort and never blocks rendering.
