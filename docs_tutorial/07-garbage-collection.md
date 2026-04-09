# Thumbnail Garbage Collection (GC)

The app stores thumbnails in `/tmp/switcher-thumbnails`. Without cleanup, stale files can accumulate.

## Source of Truth

`refresh_and_send` builds a `HashSet<String>` of active mapped client addresses from:

```bash
hyprctl clients -j
```

This set is the authoritative list for valid thumbnails in the current cycle.

## Cleanup Strategy

GC runs as a detached async task:

1. Read all files in thumbnail directory.
2. Keep files matching `<active_address>.png`.
3. Remove files for addresses not in `active_addresses`.

## Why This Is Safe

- GC is non-blocking (`tokio::spawn`).
- File operation errors are ignored intentionally (best effort).
- Rendering and input remain responsive even during directory scans.

## Practical Effect

- Lower tmpfs memory usage over time.
- No stale previews after windows close.
- No user-visible freezes from maintenance work.
