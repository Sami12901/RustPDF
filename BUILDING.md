# Building RustPdf

This document covers prerequisites, build commands, and platform targets for **RustPdf**.

---

## Prerequisites

1. **Rust Toolchain**:
   - Rust 1.95+ (recommended: the latest stable toolchain).
   - Check with `rustc --version` and `cargo --version`.
   - Install or update via [rustup.rs](https://rustup.rs):
     ```bash
     rustup update stable
     ```

2. **Operating System Dependencies**:
   - **Windows**: No additional dependencies required (uses native DirectX/WGPU and Win32 APIs).
   - **Linux**: Standard development libraries for X11/Wayland:
     ```bash
     # Ubuntu / Debian
     sudo apt-get install libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxinerama-dev libwayland-dev libxkbcommon-dev
     ```
   - **macOS**: Xcode Command Line Tools (`xcode-select --install`).

---

## Build Targets

The workspace release version is defined once in `[workspace.package].version` in `Cargo.toml`; all application and crate packages inherit it. Release builds use `--locked` so the committed `Cargo.lock` must match the dependency graph.


### 1. Desktop Application (`apps/rustpdf`)

To run in debug mode during development:
```bash
cargo run -p rustpdf
```

To build an optimized release binary:
```bash
cargo build --release -p rustpdf
```
The executable is produced at:
- `target/release/rustpdf.exe` (Windows)
- `target/release/rustpdf` (Linux / macOS)

### 2. Command Line Tool (`apps/rustpdf-cli`)

To run the CLI tool:
```bash
cargo run -p rustpdf-cli -- --help
```

To build a release binary:
```bash
cargo build --release -p rustpdf-cli
```
The executable is produced at:
- `target/release/rustpdf-cli.exe` (Windows)
- `target/release/rustpdf-cli` (Linux / macOS)

---

## Running Tests

Run the complete test suite across all 14 workspace crates:
```bash
cargo test --workspace
```

Run integration tests specifically:
```bash
cargo test --test integration_tests
```

---

## Code Quality and Lints

Check formatting:
```bash
cargo fmt --all -- --check
```

Run Clippy:
```bash
cargo clippy --workspace -- -D warnings
```
