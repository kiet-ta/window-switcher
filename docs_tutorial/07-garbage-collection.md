# Thumbnail Garbage Collection

The thumbnail directory is still `/tmp/switcher-thumbnails`.

## Source Of Truth

- Active mapped client addresses from `hyprctl clients -j`.

## Current Behavior

- GC only runs when the active-address set changes.
- Only one GC task may be in flight at a time.
- Files not matching the current `<address>.png` set are deleted.
- Errors are ignored so that rendering and input stay responsive.

This keeps tmpfs usage predictable without re-scanning the directory every refresh cycle.
