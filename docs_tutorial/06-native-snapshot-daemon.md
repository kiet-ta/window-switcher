# Native Snapshot Daemon: `wlr-screencopy`

We advanced deeply into the Wayland display mechanics, leaving simple command-line scripts (`grim`) behind in favor of a **Native Protocol Daemon**. Building an IPC daemon in Rust guarantees absolute memory safety, no hidden bottlenecks, and deterministic thumbnail pipeline routing.

## The Architectural Approach 

Wayland restricts applications from freely taking full-screen screenshots natively for security purposes (preventing generic keyloggers). To bypass this gracefully:

1. **Protocol Handshake:** We interface directly with `zwlr_screencopy_manager_v1`, a low-level extension specifically created by wlroots compositors (like Hyprland & Sway) granting authorized buffers to raw frame output.
2. **Buffer Allocation (`memfd_create`):** Wayland compositors physically map the GPU pixels directly into the client's memory address space. We constructed an abstract `SHMBuffer` using Linux's POSIX memory API natively to ensure our application receives raw ARGB bytes directly from the compositor seamlessly. Wait times effectively disappear.
3. **Rust Memory Guarantees:** Through standard RAII operations (`impl Drop`), our POSIX OS Handles (File Descriptors) implicitly destroy themselves securely when the memory block drops out of scope, eliminating memory leaks explicitly.

## Why Hook IPC Events?

Traditional switchers scan windows *after* the UI launches. This slows down the rendering. 

Instead, our daemon hooks into Hyprland's active-window shift signal (`activewindowv2`). The instant you focus out of Window A into Window B, the snapshot daemon captures Window A immediately in the background utilizing a purely isolated Tokio runtime pipe, caching it into our ultra-fast `tmpfs`.

By the time you press the actual overlay shortcut key, our window thumbnails are already loaded and waiting. The `image` crate securely handles RGBA pipeline conversions applying our optimal hardware-agnostic Triangle resampling cleanly.

> **Instructional Design Highlight:** Designing asynchronous decoupled background workers represents the pinnacle of performant Linux desktop engineering. Always assume I/O operates at peak constraints!
