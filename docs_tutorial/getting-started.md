# Getting Started 🚀

Welcome to the Hyprland Visual Window Switcher. This guide will take you from a raw codebase to a beautifully running desktop overlay in less than 2 minutes.

## 1. Prerequisites 
Ensure your Arch Linux system has the foundational development packages for GTK4 and Rust:

```bash
sudo pacman -S rustup gtk4 gtk4-layer-shell pkgconf
rustup default stable
```

*For more extensive details on why these are required, review [`02-environment-setup.md`](./02-environment-setup.md).*

## 2. Bootstrapping the Fast-Cache
Our engine achieves ultra-fast performance by leveraging your RAM rather than your SSD for image storage. Prepare the `tmpfs` directory currently hardcoded into our engine:

```bash
mkdir -p /tmp/switcher-thumbnails
rm -f /tmp/switcher-thumbnails/*
```

## 3. Build & Run
From the root of the repository, ask Cargo to pull our asynchronous workspace (`tokio`, `hyprland-rs`) and compile the executable:

```bash
# This will download the dependencies and instantly launch the UI overlay
cargo run --release
```

## 4. Usage
- Instantly observe all active window clients populated dynamically in standard GTK FlowBoxes.
- Utilize the **Arrow Keys** to surf through the 2D spatial grid.
- Press **Enter/Return** to immediately banish the UI and warp your desktop to the target window.
- Press **Escape** to cancel and return seamlessly.

## What's Next?
If you want to understand how the asynchronous thread safely passes data to GTK, jump straight into [`03-implementation-guide.md`](./03-implementation-guide.md). 

Happy hacking!
