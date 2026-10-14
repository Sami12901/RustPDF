# RustPdf — Feature Specification & Parity Matrix

**Release target:** v0.5.0. This matrix describes implemented code, not just visible menu items. OCR, genuine redaction, PDF password encryption/decryption, digital certificate signing, and file comparison remain unavailable in this release. The UI must report those limitations instead of silently claiming success.

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
| | Whole-document search | **Implemented** | Scans all pages, computes match bounds, previous/next navigation |
| | Case-sensitive & Whole-word search | **Implemented** | Exact regex-free search filters with active match highlights |
| | Copy to clipboard | **Implemented** | System clipboard copy of selected glyphs |
| | Operator-level text search & replace | **Implemented** | AST-decoded stream editing with UTF-16BE/ASCII encoding |
| | Targeted in-place text replacement | **Implemented** | Replaces specific `op_index` occurrences; preserves repeated text |
| | Overlay text replacement fallback | **Implemented** | Background-masked text block overlay with undo tracking |
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
| **Optimization** | PDF Stream Compression | **Implemented** | Flate compression of eligible uncompressed streams |
| | Object Graph Pruning | **Implemented** | Eliminates unreachable orphan objects from trailer `/Root` |
| | Image Downsampling | **Planned** | Lossy downsampling with configurable quality presets |
| **Annotations & Forms**| Text Highlight, Underline, Strikeout | **Implemented** | Standard `/Highlight`, `/Underline`, `/StrikeOut` objects |
| | Vector markup (Square, Circle, Line) | **Implemented** | Standard ISO 32000-1 geometry annotations |
| | Sticky notes (Text comments) | **Implemented** | `/Text` annotation with author and timestamp |
| | AcroForm field inspection | **Implemented** | Traverses `/AcroForm` `/Fields` (Tx, Btn, Ch) |
| | Form field editing & appearance | **Implemented** | Updates `/V`, restores `/DV`, sets `/NeedAppearances true` |
| | XFA Dynamic Forms | **Unsupported** | Displays non-destructive notice; out of scope |
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
| | User Error & Success Toasts | **Implemented** | Non-silent error reporting for save/export failures |
| | Dirty Document Indicator | **Implemented** | Unsaved dot badge and prompt on exit |
| | Bounded Transaction History | **Implemented** | Configurable depth undo/redo stack across edits |
| **Advanced Tools** | Genuine Destructive Redaction | **Unsupported** | Visual cover-ups do not remove underlying streams; genuine redaction marked unsupported |
| | Optical Character Recognition (OCR) | **Planned** | Tesseract / native OCR integration planned |
| | Digital Signatures (PKCS#7) | **Planned** | Certificate signing and validation planned |
| | File Comparison (Diff) | **Planned** | Visual page-by-page comparison planned |
| **Privacy & Security** | 100% Offline | **Implemented** | Zero telemetry, zero analytics, zero network requests |
| | Password Protection / Encryption / Decryption | **Unsupported** | No cryptographic handler is available. Protect/remove-protection actions return an explicit error and do not modify the PDF. |
| | Bounded resource limits | **Implemented** | Decompression caps, max render side limits |
| | No embedded JS execution | **Unsupported** | Untrusted scripts safely ignored |

