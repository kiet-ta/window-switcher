# Getting Started

This guide gets the current decoupled version running quickly.

## 1. Install Prerequisites

```bash
sudo pacman -S rustup gtk4 gtk4-layer-shell pkgconf
rustup default stable
```

Also ensure Hyprland CLI works:

```bash
hyprctl version
```

## 2. Prepare Runtime Cache

```bash
mkdir -p /tmp/switcher-thumbnails
rm -f /tmp/switcher-thumbnails/*
```

## 3. Build and Run

```bash
cargo run --release
```

## 4. Controls

- Arrow keys: move selection
- Enter: focus selected window and close overlay
- Escape: close overlay

## 5. Debugging

If focus dispatch fails, inspect:

```bash
tail -n 100 /tmp/window-switcher-focus.log
```

## 6. Learning Path

1. `01-architecture-concept.md`
2. `03-implementation-guide.md`
3. `06-native-snapshot-daemon.md`
4. `07-garbage-collection.md`
