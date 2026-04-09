# Implementation Guide

In this walkthrough, we will break down the implementation step-by-step. Building our window switcher involves four distinct developmental phases, transitioning from basic scaffolds to asynchronous, real-time Wayland communication.

## Phase 1: Foundation Construction
Our first step was setting up the project through Cargo. We included:
- `gtk4` and `gtk4-layer-shell` for drawing our isolated graphical overlay.
- `hyprland` to communicate with the compositor.
- `tokio` to handle asynchronous operations.

## Phase 2: Drawing the Skeleton 
In the second phase, we constructed the GTK4 user interface. The beauty of `gtk4-layer-shell` is that we can instruct the display server to treat our interface differently than regular application windows. By assigning `Layer::Overlay` and setting the keyboard mode to `Exclusive`, our menu guarantees it captures your inputs immediately without Wayland intercepting them.

We generated a `FlowBox` to arrange window thumbnails dynamically and implemented a sleek, CSS-driven "Glassmorphism" aesthetic.

## Phase 3: Spatial Navigation (Grid Math)
Because we are utilizing a grid of thumbnails, the user needs to traverse them physically. A user expects `Arrow Right` to move to the next window, and `Arrow Down` to jump to a window positioned directly underneath. 

To accomplish this:
1. We intercept the GTK keyboard events via an `EventControllerKey`.
2. Given a maximum column limit (e.g., 4 items per row), pressing "Down" translates to `current_index + 4`. 
3. Mathematically, we restrict this value so you cannot exceed the number of actively displayed windows. The UI then commands the widget at the newly calculated index to grab focus.

## Phase 4: Why Tokio and Channels? (Asynchronous Design)
In our final phase, we integrate the Hyprland IPC wrapper. This is where many desktop widgets fail. 

If we tell the GTK main thread (the thread drawing our interface) to talk to the Hyprland IPC directly, the interface will freeze while waiting for the IPC's response. This is called a **Blocking Operation**.

To prevent stuttering or freezes:
1. We launch an isolated async runtime using **Tokio**.
2. The Tokio task queries the Hyprland sockets independently.
3. We utilize **Multi-Producer, Single-Consumer (MPSC) Channels** (specifically GTK's `glib::MainContext::channel`). 

The Tokio worker effectively passes messages (like "Here is the list of open windows!") into a channel. The GTK thread listens on the other end, receiving the data smoothly in its own time. This isolates the heavy data-fetching logic entirely from the graphics generation!
