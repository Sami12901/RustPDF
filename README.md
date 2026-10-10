<p align="center">
  <img src="assets/branding/rustpdf_logo.png" alt="RustPDF Logo" width="140" style="border-radius: 16px; box-shadow: 0 4px 20px rgba(0,0,0,0.15);">
  <h1 align="center" style="font-size: 2.4rem; font-weight: 800; margin-top: 10px;">RustPDF</h1>
  <p align="center">
    <b>High-Performance, Memory-Safe PDF Workbench & Content Editor Built in Pure Rust.</b><br>
    Read, organize, combine, split, search, annotate, and visually edit PDFs in a lightning-fast, 100% offline desktop application.
  </p>
  <p align="center">
    <a href="https://github.com/Sami12901/RustPDF/actions"><img alt="CI Build Status" src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square&logo=github"></a>
    <a href="LICENSE-MIT"><img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square"></a>
    <img alt="Written in Rust" src="https://img.shields.io/badge/Rust-1.80%2B-orange?style=flat-square&logo=rust">
    <img alt="Cross-Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey?style=flat-square">
    <img alt="Zero Telemetry" src="https://img.shields.io/badge/telemetry-100%25%20offline-success?style=flat-square">
  </p>
</p>

<p align="center">
  <a href="#-key-features">Key Features</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-cli-usage">CLI Usage</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-license">License</a>
</p>

---

## 🌟 Overview

**RustPDF** is a modern, privacy-first PDF workbench designed from the ground up to replace bloated, subscription-gated PDF tools. Written entirely in pure Rust, it delivers instantaneous start-up times, minimal memory consumption, zero telemetry, and rock-solid stability.

Whether you need to merge contracts, reorganize massive scans, edit on-page text and images, apply Bates numbering, add watermarks, or automate batch pipelines via the command line, **RustPDF** gives you full control.

---

## ✨ Key Features

### ✏️ Interactive Content & Text Editing
- **Operator-Level Search & Replace**: Parse and modify PDF content stream operators (`Tj`, `TJ`) with AST decoding and proper UTF-16BE/ASCII encoding, completely avoiding raw byte splicing.
- **Targeted In-Place Editing**: Edit text blocks directly by operator index (`op_index`), preventing unintended modifications to repeated identical text strings.
- **Overlay Text Fallback**: Clean background-masked replacement tool with undo support when embedded subset fonts prevent safe in-place re-encoding.
- **Add Text**: Place custom text anywhere on any page with fine control over font size, coordinates $(X, Y)$, and RGB colors.
- **Image Insertion**: Embed PNG, JPEG, and WebP graphics into pages with custom dimension scaling, position offsets, and transparency support.
- **Watermarks**: Apply diagonal watermark stamps across documents with configurable colors, font sizes, and rotation angles.
- **Headers & Footers**: Add running headers and footers with dynamic page numbering variables (`{n}` for current page, `{total}` for total page count).
- **Backgrounds**: Tint pages or apply custom background canvas colors behind existing text and vector drawings.
- **Bates Numbering**: Legal document sequential indexing with customizable prefixes and zero-padding formats.
- **Hyperlinks**: Attach clickable URI link annotations anywhere on the page grid.

### 📄 Document Viewer & Workspace
- **Multi-Document Tabs**: Open and switch between multiple PDFs simultaneously.
- **Smooth Navigation**: Continuous vertical scrolling, single-page view, and two-page spreads.
- **Deep Zooming**: Crisp sub-pixel rendering from 20% to 800% with fit-to-width and fit-to-page shortcuts.
- **Embedded In-App Branding**: Modern custom toolbar, responsive status indicators, and SVG iconography.

### 📑 Page Organization & Thumbnail Grid
- **Visual Thumbnail Sidebar**: Reorder pages via drag-and-drop or keyboard shortcuts.
- **Page Operations**: Rotate $90^\circ$ clockwise/counter-clockwise, duplicate, delete, and insert blank pages.
- **Combine Documents**: Merge multiple independent PDFs into a single unified file.
- **Split Documents**: Decompose large documents into page ranges or single-page PDF files.

### 🗜️ PDF Compression & Optimization
- **Flate Stream Compression**: Safely compresses uncompressed or raw content streams.
- **Object Graph Pruning**: Traverses reachable objects from trailer `/Root` and eliminates unreferenced orphan objects.
- **Input/Output Ratio Reporting**: Real-time feedback on bytes saved and compression ratio.

### 🔍 Whole-Document Search & Text Extraction
- **Full-Document Search**: Scans all pages across the document with real-time match counters.
- **Match Navigator**: Visual on-canvas highlight boxes (yellow for all matches, orange for active match) with next/previous navigation.
- **Case & Word Filters**: Case-sensitive and whole-word matching filters.
- **Copy Text**: Clean Unicode text extraction preserving visual layout and reading order.

### 🎨 Modern Theming & Customization
- **Theme Modes**: Native Dark, Light, and System follow modes.
- **Color Accents**: Curated presets (Rust Orange, Ocean Blue, Emerald Green, Royal Violet, Nordic Teal) plus a live RGB picker.
- **Configurable Thumbnails**: Customizable preview card sizes, spacing density, drop shadows, and corner radii.

### 🔒 Privacy, Security & Integrity
- **100% Offline & Private**: Zero network calls, zero accounts, zero telemetry. All operations run locally on your machine.
- **Atomic Saving**: Non-destructive save operations; changes write to temporary files, verify structural validity, and atomically replace the target.
- **User-Visible File Error Reporting**: Real-time toast notifications for save, write, and permission errors instead of silent drops.
- **Bounded Undo & Redo**: Configurable-depth transaction history stack across all editing and organization operations.
- **Honest Standards Compliance**: Rejects pseudo-encryption and fake cover-up redactions in adherence to ISO 32000 principles.

---


## 📥 Downloads & Installation

Pre-compiled, standalone binaries are published for **Windows**, **Linux**, and **macOS**:

| Operating System | Package / Architecture | Direct Download | Quick Setup / Launcher |
| :--- | :--- | :--- | :--- |
| **🪟 Windows** | `x86_64` (64-bit) | [**rustpdf-windows-x86_64.zip**](https://github.com/Sami12901/RustPDF/releases/latest/download/rustpdf-windows-x86_64.zip) | Double-click [`RustPDF-Launcher.bat`](scripts/RustPDF-Launcher.bat) |
| **🐧 Linux** | `x86_64` (GLIBC 2.31+) | [**rustpdf-linux-x86_64.tar.gz**](https://github.com/Sami12901/RustPDF/releases/latest/download/rustpdf-linux-x86_64.tar.gz) | Terminal one-liner (`curl ... \| bash`) |
| **🍎 macOS** | Apple Silicon (`M1/M2/M3/M4`) | [**rustpdf-macos-aarch64.tar.gz**](https://github.com/Sami12901/RustPDF/releases/latest/download/rustpdf-macos-aarch64.tar.gz) | Terminal one-liner or extract `.tar.gz` |
| **🍎 macOS** | Intel (`x86_64`) | [**rustpdf-macos-x86_64.tar.gz**](https://github.com/Sami12901/RustPDF/releases/latest/download/rustpdf-macos-x86_64.tar.gz) | Extract `.tar.gz` and run |

---

### 🐧 Linux: One-Line Terminal Installer
Install or update RustPDF instantly in your terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/Sami12901/RustPDF/main/scripts/install.sh | bash
```
* **Location**: Installs standalone `rustpdf` and `rustpdf-cli` into `~/.local/bin/`.
* **Desktop App**: Automatically registers `rustpdf.desktop` in your application launcher.
* **Auto-Fallback**: If prebuilt binaries are unavailable, it will automatically build via Cargo.

---

### 🪟 Windows: One-Line PowerShell Setup & Launcher
Open PowerShell and run:
```powershell
irm https://raw.githubusercontent.com/Sami12901/RustPDF/main/scripts/install.ps1 | iex
```
* **Desktop Shortcut**: Automatically creates a `RustPDF` desktop shortcut.
* **PATH Setup**: Adds `rustpdf` and `rustpdf-cli` to your user PATH.
* **Launcher Script**: You can also double-click [**`scripts/RustPDF-Launcher.bat`**](scripts/RustPDF-Launcher.bat) at any time to run the app.

---

### 📦 Cargo Package Manager (Cross-Platform)
If you have the Rust toolchain installed, you can compile and install directly:
```bash
cargo install --git https://github.com/Sami12901/RustPDF.git rustpdf rustpdf-cli
```

---

## 🚀 Quick Start

### Prerequisites
- [Rust 1.80+](https://rustup.rs/) (stable toolchain)
- Cargo package manager

### Run Desktop GUI
```bash
# Clone the repository
git clone https://github.com/Sami12901/RustPDF.git
cd RustPDF

# Run the desktop application
cargo run --release -p rustpdf
```

### Build Binaries
```bash
# Build optimized release binaries for GUI and CLI
cargo build --release
```
The compiled binaries will be located at:
- **GUI Application**: `target/release/rustpdf.exe` (Windows) or `target/release/rustpdf` (Linux/macOS)
- **CLI Utility**: `target/release/rustpdf-cli.exe` (Windows) or `target/release/rustpdf-cli` (Linux/macOS)

---

## 💻 CLI Usage

RustPDF includes a dedicated command-line tool `rustpdf-cli` for terminal workflows and automated processing:

```bash
# Inspect PDF metadata and page count
rustpdf-cli info document.pdf

# Render a page to an image
rustpdf-cli render document.pdf --page 1 --out page_1.png

# Combine multiple PDFs
rustpdf-cli combine part1.pdf part2.pdf part3.pdf --out merged.pdf

# Split a PDF into chunks of 5 pages
rustpdf-cli split document.pdf --every 5 --out-dir ./output/

# Extract plain text from all pages
rustpdf-cli extract document.pdf --out document.txt
```

---

## 🏗️ Architecture

The workspace is organized into modular, clean-room crates designed for maintainability and testing:

```
RustPDF/
├── apps/
│   ├── rustpdf/              # Desktop GUI application (egui + eframe)
│   └── rustpdf-cli/          # Command-line utility
├── crates/
│   ├── rustpdf-annot/        # PDF annotations & highlights
│   ├── rustpdf-automation/   # Headless batch automation runner
│   ├── rustpdf-cos/          # Low-level Carousel Object Structure (COS) & security
│   ├── rustpdf-edit/         # Content editing (Text, Images, Watermarks, History)
│   ├── rustpdf-engine/       # Central DocumentSession facade & undo/redo
│   ├── rustpdf-export/       # Blank PDF generator, plain text & JSON export
│   ├── rustpdf-filters/      # Standard PDF stream filters (Flate, ASCIIHex, etc.)
│   ├── rustpdf-fonts/        # Standard 14 font metrics & sizing
│   ├── rustpdf-forms/        # AcroForms reading, filling, and export
│   ├── rustpdf-geom/         # 2D geometry (Point, Rect, Matrix transform)
│   ├── rustpdf-model/        # High-level document abstractions
│   ├── rustpdf-organize/     # Page manipulation (reorder, rotate, split, merge)
│   ├── rustpdf-render/       # CPU page rasterization & text extraction (hayro)
│   └── rustpdf-ui/           # egui UI components, icons, modals, and viewer
└── assets/
    ├── branding/             # RustPDF logos and banners
    └── icons/                # High-DPI professional vector SVG icons
```

---

## 🧪 Testing

Run all unit, integration, and roundtrip tests across the entire workspace:

```bash
cargo test --workspace
```

All 20+ unit tests across parsing, geometric math, stream filtering, session history, and CLI workflows pass out-of-the-box.

---

## 📚 Documentation

Detailed developer and user manuals are available in the repository:

- 📖 [User Guide](USER_GUIDE.md) — Complete guide on viewer controls, keyboard shortcuts, and tools.
- 🛠️ [Building Guide](BUILDING.md) — Platform-specific build steps and packaging.
- 📐 [Architecture](ARCHITECTURE.md) — Deep dive into engine design, concurrency, and data models.
- 📋 [Feature Matrix](FEATURES.md) — Comparison against standard PDF specifications.
- 🔒 [Security Policy](SECURITY.md) — Sandbox model, parser memory safety, and reporting.
- 🤝 [Contributing](CONTRIBUTING.md) — How to submit pull requests and code conventions.

---

## 👥 Authors & Acknowledgments

- **Sami Islam** ([@Sami12901](https://github.com/Sami12901)) — Project Lead & Creator
- Built with the fantastic Rust ecosystem: [`egui`](https://github.com/emilk/egui), [`hayro`](https://github.com/LaurenzV/hayro), [`lopdf`](https://github.com/J-F-Liu/lopdf), and [`image`](https://github.com/image-rs/image).

---

## 📜 License

Dual-licensed under either:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
