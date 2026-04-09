# Environment Setup

This chapter prepares your machine for the current panic-safe architecture.

## 1. Install System Packages

```bash
sudo pacman -S rustup gtk4 gtk4-layer-shell pkgconf
rustup default stable
```

`pkgconf` is required so Rust can link GTK system libraries.

## 2. Verify Hyprland CLI Availability

The backend now depends on native `hyprctl` commands for state fetching:

```bash
hyprctl version
hyprctl clients -j | head -c 120
```

If these fail, the app can start but window data updates will silently skip.

## 3. Prepare Thumbnail Cache (tmpfs)

```bash
mkdir -p /tmp/switcher-thumbnails
rm -f /tmp/switcher-thumbnails/*
```

`/tmp` is usually tmpfs on modern Linux, so thumbnail reads stay fast.

## 4. Confirm Rust Dependencies

Your `Cargo.toml` should include:

- `gtk4`
- `gtk4-layer-shell`
- `hyprland` (only for `EventListener`)
- `tokio`
- `serde` + `serde_json`

## 5. Build Check

```bash
cargo check
```

## 6. Runtime Diagnostics

Focus-dispatch worker errors are written to:

```text
/tmp/window-switcher-focus.log
```

Useful commands:

```bash
tail -f /tmp/window-switcher-focus.log
```
