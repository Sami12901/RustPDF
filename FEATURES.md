# RustPdf — Feature Specification & Parity Matrix

This document provides a comprehensive catalogue of all capabilities in **RustPdf** compared to standard PDF workbench requirements.

---

## Parity Classification
- **Implemented**: Fully implemented with independent, working code and tests.
- **Partially Implemented**: Core logic implemented; advanced edge cases in progress.
- **Planned**: Scheduled for future milestones.
- **Unsupported**: Explicitly out of scope (e.g. proprietary or obsolete standards).

---

## Feature Matrix

| Feature Area | Capability | Status in RustPdf | Implementation Details |
| :--- | :--- | :--- | :--- |
| **Viewing & Display** | Asynchronous page rasterization | **Implemented** | Bounded multi-worker thread pool in `rustpdf-render` |
| | High-DPI subpixel zoom | **Implemented** | Continuous scaling from 20% to 800% with clamping |
| | Single-Page View | **Implemented** | Focuses single page in view |
| | Continuous Vertical Scroll | **Implemented** | Smooth scroll container with viewport culling |
| | Two-Page Facing Spread | **Implemented** | Dual-spread layout |
| | Viewport Page Rotation | **Implemented** | 90° increments (0°, 90°, 180°, 270°) |
| | Multi-document tabs | **Implemented** | Tab bar with dirty-state indicator and close handling |
| **Text & Search** | Unicode glyph text extraction | **Implemented** | `hayro::interpret_page` with custom `TextDevice` |
| | Reading-order text selection | **Implemented** | Baseline grouping and word spacing reconstruction |
| | Find as you type | **Implemented** | Real-time highlight boxes over matched glyphs |
| | Case-sensitive & Whole-word search | **Implemented** | In-document search filters with match counter |
| | Copy to clipboard | **Implemented** | System clipboard copy of selected glyphs |
| **Page Organization** | Thumbnail Card Grid | **Implemented** | Multi-select grid of rendered PDF pages |
| | Multi-page selection | **Implemented** | Single, Ctrl-toggle, and Shift-range selection |
| | Physical Page Rotation | **Implemented** | Modifies `/Rotate` attribute in page dictionary |
| | Page Deletion | **Implemented** | Deletes pages while preserving >= 1 page invariant |
| | Page Duplication | **Implemented** | Deep-copies page object and stream references |
| | Blank Page Insertion | **Implemented** | Inserts standard (Letter, A4) blank pages |
| | Page Reordering | **Implemented** | Rebuilds page tree Kids array |
| **Document Assembly** | Combine multiple PDFs | **Implemented** | Merges multiple PDFs with object renumbering |
| | Split by N pages | **Implemented** | Chunks document into N-page segment files |
| | Split into single pages | **Implemented** | Decomposes document into individual page files |
| | Extract selected pages | **Implemented** | Extracts chosen pages to fresh standalone PDF |
| **Annotations & Forms**| Text Highlight, Underline, Strikeout | **Implemented** | Standard `/Highlight`, `/Underline`, `/StrikeOut` objects |
| | Vector markup (Square, Circle, Line) | **Implemented** | Standard ISO 32000-1 geometry annotations |
| | Sticky notes (Text comments) | **Implemented** | `/Text` annotation with author and timestamp |
| | AcroForm field inspection | **Implemented** | Traverses `/AcroForm` `/Fields` (Tx, Btn, Ch) |
| | Form field editing & reset | **Implemented** | Updates `/V` value and restores default `/DV` |
| | XFA Dynamic Forms | **Unsupported** | Displays warning notice; out of scope |
| **Appearance & Theme** | Dark / Light / System Mode | **Implemented** | Reactive theme switching |
| | 6 Accent Presets + Custom Color | **Implemented** | Rust Orange, Blue, Green, Violet, Teal, Custom |
| | Full RGB Color Picker | **Implemented** | Interactive color picker in settings |
| | Semantic Design Tokens | **Implemented** | Background, panel, surface, border, text tokens |
| | Theme Import / Export (JSON) | **Implemented** | Shareable theme preset serialization |
| | Theme Persistence | **Implemented** | Persisted across application restarts |
| **Thumbnails Engine** | Configurable thumbnail size | **Implemented** | 80pt to 320pt card scaling |
| | Grid density & sidebar width | **Implemented** | Dynamic padding and sidebar adjustment |
| | Corner radius & drop shadow toggles | **Implemented** | Card aesthetics customization |
| | Layout Presets | **Implemented** | Compact, Balanced, and Large Previews |
| | Thumbnail Persistence | **Implemented** | Persisted across application restarts |
| **Safe Saving & History**| Atomic Save | **Implemented** | Temp file write + sync + validation + atomic rename |
| | Dirty Document Indicator | **Implemented** | Unsaved dot badge and prompt on exit |
| | Deep Undo / Redo | **Implemented** | Transaction stack for all page & metadata operations |
| **CLI & Automation** | `rustpdf-cli info` | **Implemented** | Metadata, version, and page count summary |
| | `rustpdf-cli text` | **Implemented** | Plain text extraction |
| | `rustpdf-cli render` | **Implemented** | Page rasterization to PNG |
| | `rustpdf-cli combine / split` | **Implemented** | Headless document merging and splitting |
| | Typed JSON automation | **Implemented** | Safe local headless execution engine |
| **Privacy & Security** | 100% Offline | **Implemented** | Zero telemetry, zero analytics, zero network requests |
| | Bounded resource limits | **Implemented** | Decompression caps, max render side limits |
| | No embedded JS execution | **Implemented** | Untrusted scripts safely ignored |
