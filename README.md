<div align="center">
  <h1>🌌 Hyprland Visual Window Switcher</h1>
  <p><b>A lightning-fast, glassmorphism-styled window switcher for Hyprland built in Pure Rust.</b></p>
  
  [![Rust Build](https://github.com/kiet-ta/window-switcher/actions/workflows/rust.yml/badge.svg)](https://github.com/kiet-ta/window-switcher/actions)
  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
  [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/kiet-ta/window-switcher/pulls)

</div>

## 📖 Overview

The **Hyprland Visual Window Switcher** provides a premium, visually engaging replacement for `rofi` or `wofi` when swapping between active windows. Leveraging `gtk4-layer-shell`, `tokio`, and `hyprland-rs`, this switcher generates a gorgeous overlay independent of generic Wayland tiling protocols.

It features **Snapshot Caching** via an in-memory `tmpfs` disk and **Asynchronous Thread Isolation**, ensuring your UI remains silky smooth at 60 FPS without ever staggering.

## ✨ Features

- **Zero-Latency UI:** Asynchronous fetching handles IPC without blocking the UI thread.
- **Pure Glassmorphism:** Sleek, customizable GTK4 CSS styling out of the box.
- **Sub-Millisecond Read Times:** Reads window thumbnails explicitly from a RAM disk.
- **Spatial Navigation:** 2D native keyboard movement mathematically bound to a grid structure.

## 🚀 Installation & Deployment

### Pre-compiled Binary (Arch Linux)

The fastest way to install is by downloading the native binary directly from our [Releases Page](https://github.com/kiet-ta/window-switcher/releases).

1. Download the latest `window-switcher` binary.
2. Mark it executable: `chmod +x window-switcher`
3. Move it to your path: `sudo mv window-switcher /usr/local/bin/`

### Compiling from Source (AUR / PKGBUILD)

For Arch Linux users, we provide a native `PKGBUILD` ensuring exact tracking natively via `pacman`.

```bash
git clone https://github.com/kiet-ta/window-switcher.git
cd window-switcher/packaging
makepkg -si
```

## 🎮 Usage

Once installed, you can launch the window switcher by substituting your preferred hotkey in the Hyprland configuration (`~/.config/hypr/hyprland.conf`).

For example, to map the switcher to `SUPER + TAB`:

```conf
bind = SUPER, tab, exec, window-switcher
```

## 📚 Getting Started

Ready to compile manually or modify the internal Rust logic? Head over to the [Getting Started Guide](./docs_tutorial/getting-started.md) to bootstrap your development environment.

Our internal logic and structural guidelines are thoroughly documented in the `docs_tutorial/` folder.

## 🤝 Open Source & Community

We firmly believe in building together. We welcome contributions, from hotfixes to major architectural overhauls!

- Please read our [**Contributing Guidelines**](./CONTRIBUTING.md) to learn how you can submit Pull Requests.
- For our strict vulnerability and bug-bounty policies, review [**SECURITY.md**](./SECURITY.md).

## 💖 Support and Sponsorship

If you love this tool and it saves you time every day, consider supporting its active development via our [**Funding and Sponsors page**](/.github/FUNDING.yml).
