# Hyprland Visual Window Switcher

A GTK4 overlay switcher for Hyprland, written in Rust, with MRU Alt-Tab behavior, debounced compositor refreshes, and cached thumbnails.

## What Changed In vNext

- Event-driven backend refresh with a `75ms` debounce and a `3s` safety poll instead of a `250ms` hot-path polling loop.
- Snapshot-based UI updates: the GTK layer now receives `UiSnapshot` payloads and updates cards in place instead of rebuilding the entire grid every refresh.
- MRU ordering with correct initial selection: when there are at least two windows, the switcher preselects the previous window, matching the expected Alt-Tab mental model.
- Dedicated focus and thumbnail workers: focus dispatch is queued off the UI thread, and thumbnail capture runs in a reusable screencopy session.
- Explicit UI states for loading, empty results, preview loading, preview missing, and preview failure.

## Runtime Model

The app keeps the existing CLI surface:

```bash
window-switcher
window-switcher --daemon
```

- `--daemon` starts the overlay hidden.
- `SIGUSR1` toggles visibility for the running daemon instance.
- The backend stays Hyprland/Wayland-only in this phase.

## Controls

- `Tab` / `Right`: move forward
- `Left`: move backward
- `Up` / `Down`: move vertically in the current responsive grid
- `Enter`: focus the selected window and close the overlay
- `Escape`: close the overlay
- release `Alt`: focus the currently selected window and close the overlay

## Architecture

### Backend

- `hyprctl clients -j` is the main source of truth for visible windows.
- `hyprctl activewindow -j` is parsed once per refresh and supports both `monitor` and `monitorID`.
- Monitor names are cached from `hyprctl monitors -j` and refreshed on startup, cache miss, or every `10s`.
- The backend emits `UiSnapshot { items, selected_address, status, revision }` only when the rendered content actually changes.

### Workers

- Focus worker: receives `FocusCommand`, executes `hyprctl dispatch focuswindow`, and logs failures to `/tmp/window-switcher-focus.log`.
- Thumbnail worker: receives `ThumbnailJob`, reuses a Wayland screencopy session, coalesces in-flight jobs per address/monitor pair, and writes thumbnails to `/tmp/switcher-thumbnails/<address>.png`.

### UI

- The GTK layer caches cards by window address.
- Order changes reuse existing widgets instead of recreating them.
- Selection is tracked by `selected_address`, not by a stale index.
- Cards show active-window state separately from selected-target state.

## Development

### Requirements

- Rust stable toolchain
- `gtk4`
- `gtk4-layer-shell`
- `wayland`
- a Hyprland session with working `hyprctl`

### Run

```bash
cargo run --release
```

### Checks

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release --locked
```

## Packaging

The Arch PKGBUILD remains under [`packaging/PKGBUILD`](./packaging/PKGBUILD).

## Limitations

- This phase still uses monitor/workspace previews rather than true per-window cropped thumbnails.
- Final validation should happen on a Linux machine with the Rust toolchain available; the current workspace did not expose `cargo` during implementation.
