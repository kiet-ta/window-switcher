# Environment Setup

Before writing our code, we must construct a solid foundation. In this guide, we will set up the Rust environment, install GTK4 system libraries, and create a `tmpfs` RAM disk on Arch Linux to cache our thumbnail images.

## Step 1: Install Rust and GTK4 Libraries
Our project relies on GTK4 to draw the graphical interface and Layer Shell to position it explicitly over the Hyprland desktop. On Arch Linux, everything runs through `pacman`.

Open your terminal and run the following command to grab the necessary development packages:

```bash
sudo pacman -S rustup gtk4 gtk4-layer-shell pkgconf
```

Once installed, ensure your Rust toolchain is set to stable:

```bash
rustup default stable
```

> [!TIP]
> The `pkgconf` tool is **imperative**! It is a helper that allows the Rust compiler to find where the C-based GTK4 libraries are located on your system. Without it, the build will fail immediately.

## Step 2: Creating the Tmpfs (RAM Disk)
We need a specialized memory location to store our window screenshots. A `tmpfs` is perfect because it acts like a folder, but it is stored identically in RAM, making it extremely fast.

We will create a mount point in the `/tmp` folder. By default in Arch Linux, `/tmp` is already mounted as a `tmpfs`. However, it's good practice to create a dedicated sub-folder so our application data stays organized.

Run these commands:
```bash
# Create the directory
mkdir -p /tmp/switcher-thumbnails

# Ensure the folder is clear if you're restarting the app
rm -f /tmp/switcher-thumbnails/*
```

Because `/tmp` natively utilizes `tmpfs` in modern systemd-based Linux distributions, we don't need to configure `/etc/fstab`. Any image placed inside `/tmp/switcher-thumbnails` will naturally benefit from RAM-level speeds.

## Step 3: Project Scaffold Check
At this stage, your `Cargo.toml` file should already contain our project dependencies:
- `gtk4`
- `gtk4-layer-shell`
- `hyprland`
- `tokio`

You can verify that your operating system can link everything together by running a quick compilation check in your terminal:

```bash
cargo check
```

If it successfully finishes without errors, your foundation is complete! We are now ready to write the UI skeleton in our next chapter.
