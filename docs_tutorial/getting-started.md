# Getting Started

## 1. Install Prerequisites

```bash
sudo pacman -S rustup gtk4 gtk4-layer-shell pkgconf
rustup default stable
```

Verify Hyprland CLI access:

```bash
hyprctl version
```

## 2. Prepare Runtime Directories

```bash
mkdir -p /tmp/switcher-thumbnails
```

## 3. Run Locally

```bash
cargo run --release
```

For daemon mode:

```bash
cargo run --release -- --daemon
```

Then toggle the running daemon with `SIGUSR1`.

## 4. Validate Locally

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release --locked
```

## 5. Recommended Reading

1. `01-architecture-concept.md`
2. `03-implementation-guide.md`
3. `06-native-snapshot-daemon.md`
4. `07-garbage-collection.md`
