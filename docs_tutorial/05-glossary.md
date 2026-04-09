# Glossary

- **Active Window Signal:** A compositor event that indicates focus changed to another window.
- **Bounded Queue:** A channel with a maximum capacity; protects memory and backpressure behavior.
- **Compositor:** The Wayland process (Hyprland) that manages surfaces, focus, and rendering.
- **Decoupled Architecture:** Design where UI, event signaling, data fetch, and action dispatch are separated into independent components.
- **EventListener (`hyprland-rs`):** Lightweight listener used here only to receive events, not to fetch structured state.
- **Failover (Silent):** Error handling strategy where a failed operation is skipped without crashing the app.
- **Focus Dispatch Worker:** Background thread that runs `hyprctl dispatch focuswindow ...` outside the GTK thread.
- **GTK Main Thread:** UI thread responsible for drawing widgets and processing keyboard events.
- **Hyprctl JSON API:** CLI interface (`hyprctl ... -j`) returning JSON for clients, active window, and monitors.
- **Observability:** Ability to inspect runtime behavior through logs and status output.
- **Serde:** Rust framework for serialization/deserialization.
- **Tmpfs:** RAM-backed filesystem used for thumbnail cache (`/tmp/switcher-thumbnails`).
- **Worker Thread:** Dedicated non-UI thread for I/O-heavy or blocking operations.
