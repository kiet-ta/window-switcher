# Window Switcher Context (AGENT.md)

## 1. Project Overview
**Name:** Hyprland Visual Window Switcher
**Description:** A GTK4 overlay switcher for the Hyprland Wayland compositor, written in Rust. It provides an MRU (Most Recently Used) Alt-Tab behavior, responsive 2D grid navigation, debounced compositor refreshes, and native Wayland cached thumbnails.

## 2. Architecture & Concurrency Model
The application strictly separates the UI thread from the backend workers to ensure high performance and zero UI stuttering.

- **Main Thread (GTK):** Handles rendering, CSS styling, and keyboard input. Receives state updates via asynchronous channels.
- **Tokio Runtime:** An asynchronous background orchestrator manages timers, debouncing, and listens for Hyprland IPC events.
- **Worker Threads (std::thread):** 
  - **Focus Worker:** Executes `hyprctl dispatch focuswindow` to avoid blocking.
  - **Thumbnail Worker:** Interacts with the Wayland `wlr-screencopy-unstable-v1` protocol to capture active workspaces.
- **Communication:** Uses `async_channel` for robust, thread-safe message passing between the backend orchestrator, workers, and the GTK frontend.

## 3. Directory Structure & Key Modules

### `src/` Root
- `main.rs`: Application entry point. Bootstraps GTK, sets up the `SIGUSR1` listener for daemon toggling, and initializes the local tmpfs cache.
- `config.rs`: Centralized configuration containing all constants, magic numbers, UI dimensions, debounce timings, and queue capacities.

### `src/backend/` (Data & System Integration)
- `hyprctl.rs`: The brain of the backend. Polls and listens to Hyprland (`clients -j`, `activewindow -j`, `monitors -j`). Manages MRU ordering, UI snapshot generation (`UiSnapshot`), and thumbnail GC (Garbage Collection).
- `screencopy.rs`: Native Wayland implementation using `wayland-client`. Captures frames directly into shared memory (`memfd`) using the `wlr-screencopy` protocol.
- `image_processor.rs`: Takes raw RGBA buffers from screencopy, resizes them efficiently using `image::imageops::resize`, and encodes them to `.png`.

### `src/ui/` (Presentation Layer)
- `mod.rs`: Builds the GTK UI. Subscribes to `UiSnapshot` payloads. Crucially, it updates `WindowCard` widgets **in-place** (caching them by window address) instead of rebuilding the UI grid on every refresh.
- `input.rs`: Handles spatial keyboard navigation (Up, Down, Left, Right) and standard Alt-Tab/Escape workflows. Captures `EventControllerKey` at the `Capture` phase.
- `css.rs`: Inline CSS providing the "vibrant, glassmorphism, dynamic" GTK styling with CSS transitions.

## 4. Data Flow
1. **Trigger:** User switches windows, or a safety timer ticks. `backend/hyprctl.rs` listens via `hyprland-rs` EventListener.
2. **Debounce:** Events are debounced for `REFRESH_DEBOUNCE_MS` (75ms).
3. **Fetch & Parse:** Backend executes `hyprctl` commands with transient error retry logic.
4. **State Calculation:** The backend computes MRU sequence, resolves monitor outputs, checks existing thumbnails, and requests new thumbnails if the active window changed.
5. **Emit:** An immutable `UiSnapshot` is emitted to the GTK main thread.
6. **Render:** `ui/mod.rs` calculates diffs (adds/removes/reorders cards) and updates GTK widget properties (labels, CSS classes, images).

## 5. Coding Standards & Conventions
- **Asynchronous UI Updates:** Never block the main GTK loop (`glib::MainContext::default().spawn_local` is used for async loops on the UI thread).
- **Graceful Degradation:** If `hyprctl` fails or thumbnails aren't ready, the UI explicitly shows loading states (`Preview loading`, `Preview unavailable`) and uses a fallback icon instead of crashing.
- **In-Place UI Updates:** Recreating GTK widgets is expensive. Always update existing widgets where possible.
- **Memory Management:** Thumbnail temp files are periodically cleaned up by an async GC task in `hyprctl.rs`.
- **Global Rules Adherence:** 
  - All variables, comments, and commit messages are in English.
  - Errors are handled properly; avoid `unwrap()` or empty catches unless technically justified. Log errors (e.g., focus failures are logged to `/tmp/window-switcher-focus.log`).
  - Architecture follows single-responsibility (backend vs ui vs image processing).
