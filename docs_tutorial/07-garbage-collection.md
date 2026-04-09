# Thumbnail Garbage Collection (GC)

After building the Native Snapshot Daemon in Phase 2, we inevitably introduced a caching problem. The switcher caches thumbnails to `/tmp/switcher-thumbnails` rapidly, but as windows close, these image files pile up infinitely silently consuming valuable `tmpfs` RAM. 

## Design Pattern: Fire-and-Forget GC

To keep the application mathematically bounded and strictly SOLID, we decoupled the Garbage Collector from the core render pipeline.

### Implementation Concept
1. **Source of Truth:** The `refresh_and_send` function continuously maps currently active, mapped windows. We extract exactly this active list into an optimal `HashSet<String>` containing the raw native `hex` representation strings of active addresses.
2. **Asynchronous Spawning:** Executing `tokio::spawn(async move { ... })` creates a secondary background thread natively pushing the heavy Linux POSIX filesystem calls perfectly out of the event listener boundaries.
3. **Difference Mapping:** The GC parses `/tmp/switcher-thumbnails`. For every target file structured as `<address>.png`, we parse the prefix. If the prefix string fundamentally does **not** exist in the known-good active `HashSet`, the underlying memory file is destroyed instantly using `tokio::fs::remove_file`.

```rust
// A structural subset highlighting O(1) mathematical lookup mappings
let prefix = name_str.trim_end_matches(".png");
if !active_addresses.contains(prefix) {
    let _ = tokio::fs::remove_file(entry.path()).await;
}
```

> **Instructional Highlight (KISS & Error Handling):** Standard Unix systems often lock operations. Deliberately suppressing edge exceptions on file deletions (`let _ = tokio...`) acknowledges that if a file is externally locked or suddenly vanished during traversal, the GC strictly ignores it and repeats on the next pulse. It avoids crashing the pipeline over non-fatal latency events!
