# Native Snapshot Daemon (`wlr-screencopy`)

This subsystem captures thumbnails of the previously active window/workspace in the background.

## Current Trigger Model

1. `EventListener` emits an active-window-change signal.
2. Backend reads the previous `last_state` (`address`, `monitor`).
3. Backend calls `capture_active_workspace(out_path, monitor_name).await`.

This keeps screenshot work out of the GTK rendering path.

## Why It Matters

- UI can open instantly because thumbnails are often pre-captured.
- Heavy screencopy and image processing stay off the keyboard/UI thread.
- Failures in capture do not terminate the app (`let _ = ...` best effort).

## Relation to Decoupled Fetching

The screenshot daemon no longer depends on `hyprland::data` structs.  
`last_state` is built from:

- `hyprctl activewindow -j` (address + monitor id)
- `hyprctl monitors -j` (monitor id -> monitor name)

This aligns screenshot routing with the same panic-safe data model used for the window list.

## Operational Notes

- Output path format: `/tmp/switcher-thumbnails/<address>.png`
- Capture is asynchronous and fire-and-forget.
- Missing monitor mapping simply skips that capture cycle (safe degradation).
