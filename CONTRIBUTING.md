# Contributing to Hyprland Window Switcher

First off, thank you for considering contributing to the Hyprland Visual Window Switcher! It's people like you that make the open-source Linux community such an incredible place to explore and build.

## Developer Guidelines
Before you submit a Pull Request, please ensure you adhere to our explicit Engineering Standards:

1. **English Only:** All code comments, JSDoc/RustDoc bindings, function names, and commit messages must be explicitly written in English.
2. **Security First:** Protect against user input. Do not arbitrarily shell out to `bash -c` unless validated rigorously. 
3. **Rust Styling:** Your code must pass `cargo fmt` and `cargo clippy -- -D warnings`.

## Pull Request Process
1. **Fork the repository** and create your branch from `main`.
2. **Write Unit Tests** for any new IPC parsing logic or Grid Math implementations.
3. **Commit Standards:** We enforce Conventional Commits. Your commit must look like:
   - `feat: implement grim screenshot pipeline`
   - `fix: resolve out-of-bounds error in grid math`
   - `docs: update tmpfs prerequisite in getting started`

## Setting Up Your Environment
To get up and running, please read:
- `docs_tutorial/02-environment-setup.md`
- `docs_tutorial/getting-started.md`

We actively review PRs weekly. Welcome aboard!
