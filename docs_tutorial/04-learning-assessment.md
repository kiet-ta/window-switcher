# Learning Assessment

Welcome to the Instructional Design Document (IDD) checkpoint. After completing the four code phases, please reflect on these scenarios to assess your system engineering mindset.

## Reflective Questions 🧠

**1. What happens if the `tmpfs` gets full?**
> **Scenario:** You open 500 windows at once, and the screenshot daemon fills `/tmp/switcher-thumbnails` entirely. Because `/tmp` lives in RAM, will your computer crash?
> **Answer Insight:** Modern Linux kernels limit `tmpfs` to roughly 50% of your total RAM by default. Your computer won't crash immediately, but the switcher will fail to write new thumbnails, throwing an error. Good code should monitor disk space and delete old thumbnails!

**2. Why did we use `gtk4-layer-shell` instead of standard `gtk4` windows?**
> **Scenario:** Wayland is heavily restricted for security purposes. If you build a standard GTK interface, Hyprland will wrap it in a window border and subject it to tiling rules.
> **Answer Insight:** Layer Shell protocols literally bypass the tiling compositor logic, allowing panels, switchers, and widgets to draw directly on top of the screen as "Overlays", completely ignoring desktop rules.

**3. Why did we implement Tokio channels instead of blocking the main thread?**
> **Scenario:** Imagine if a script fetching Hyprland window clients stalls for 2 seconds. What happens to the UI?
> **Answer Insight:** If the GTK thread fetches the data directly, the entire application freezes (stutters) for 2 seconds. The user's keystrokes are ignored. Offloading this to Tokio guarantees that the UI stays fluid at 60+ FPS while waiting for the data to arrive asynchronously.

**4. How does the 2D Grid Math protect against "Out of Bounds" errors?**
> **Scenario:** You are on the last column mapping out a down-press logic: `index + 4`. The total array is only 5 items.
> **Answer Insight:** Our math explicitly compares the calculated offset against the maximum array size. If `index + 4` exceeds the available limit, we forcefully anchor (snap) the selection to the final available item `total - 1`.
