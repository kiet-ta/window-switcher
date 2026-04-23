# Architecture Concept

The current implementation is built around a snapshot pipeline:

1. Hyprland emits active-window change events.
2. The backend debounces those bursts for `75ms`.
3. The backend fetches the latest client list and active window state.
4. A `UiSnapshot` is emitted only if the rendered result changed.
5. GTK updates existing cards in place.

## Why This Matters

- The UI thread does not spawn compositor commands directly.
- The overlay no longer rebuilds the full grid on every refresh.
- Keyboard selection is tracked by address, which removes stale-index bugs.
- Focus dispatch and thumbnail capture run in dedicated workers.

## Main Components

| Component | Responsibility |
| --- | --- |
| `src/backend/hyprctl.rs` | Debounced Hyprland refresh loop, MRU ordering, focus worker, GC gating |
| `src/backend/screencopy.rs` | Reusable Wayland screencopy session |
| `src/backend/image_processor.rs` | BGRA -> RGBA conversion, aspect-ratio resize, deduplicated PNG writes |
| `src/ui/mod.rs` | Snapshot rendering, card cache, loading/empty/degraded states |
| `src/ui/input.rs` | Address-based keyboard navigation and focus dispatch |

## Output Contract

The backend emits:

```rust
UiSnapshot {
    items: Vec<WindowData>,
    selected_address: Option<String>,
    status: UiStatus,
    revision: u64,
}
```

This makes rendering deterministic and easy to diff.
