# RustPdf — Compatibility Matrix & Standard Support

This document specifies the technical compatibility, ISO 32000 conformance, platform targets, and limitations of **RustPdf v0.5.0**.

The v0.5.0 audit hardens page-order operations, validates extraction ranges, preserves inherited page attributes when normalizing nested page trees, and refuses unsafe pseudo-decryption. OCR, destructive redaction, encryption/decryption, certificate signing, and file comparison remain unsupported.

---

## 1. Conformance Status Definitions

- **Implemented**: Implemented, integrated, and verified with unit and integration tests.
- **Tested**: Verified with automated tests and real PDF document fixtures.
- **Partially Supported**: Core functionality operates; specific complex edge cases remain in development.
- **Planned**: Architecturally roadmaped for upcoming releases.
- **Unsupported**: Explicitly not supported in the current release.

---

## 2. ISO 32000 & PDF Specification Conformance

| Feature / Standard Area | Status | Technical Details & Limitations |
| :--- | :--- | :--- |
| **PDF 1.0 – 1.7 Reading** | **Tested** | Parses cross-reference tables, object streams, catalogs, pages, and annotations via `lopdf` and `rustpdf-cos`. |
| **PDF 2.0 (ISO 32000-2)** | **Partially Supported** | Supports common PDF 2.0 document structures, but does not support UTF-8 string literals or modern cryptographic handlers. |
| **Linearized PDFs ("Fast Web View")** | **Partially Supported** | Reads and renders correctly; written files are standard incremental/regular PDFs, not linearized. |
| **Encrypted PDFs (Standard Security)** | **Unsupported** | Password validation, RC4/AES key derivation, and string/stream encryption are not yet implemented. Document protection is disabled to avoid pseudo-security. |
| **Public-Key / Certificate Security** | **Planned** | PKCS#7 digital signature validation and certificate encryption are planned for a future release. |
| **AcroForm Interactive Forms** | **Partially Supported** | Reads form fields (`Tx`, `Btn`, `Ch`); updates values (`/V`) and sets `/NeedAppearances true` for viewer rendering. |
| **XFA Dynamic Forms** | **Unsupported** | XML Forms Architecture is proprietary and obsolete; a non-destructive notice is shown. |
| **Embedded JavaScript (`/JS`)** | **Unsupported** | Embedded scripting execution is intentionally disabled for security. |

---

## 3. Content Stream & Text Editing Compatibility

| Operator / Capability | Status | Handling Details |
| :--- | :--- | :--- |
| **Text Showing (`Tj`, `'`, `"`)** | **Tested** | Decoded and parsed into AST operations; literal strings replaced without corrupting operands. |
| **Kerning Arrays (`TJ`)** | **Tested** | Parses array elements (strings and kerning offsets); strings replaced cleanly. |
| **Text Positioning (`Tm`, `Td`, `TD`, `T*`)** | **Tested** | Coordinates extracted to calculate bounding boxes; preserved during editing. |
| **Encoding: Standard ASCII** | **Tested** | Full roundtrip literal string encoding. |
| **Encoding: Unicode (UTF-16BE BOM)** | **Tested** | Non-ASCII strings encoded with standard `\xFE\xFF` UTF-16BE literals. |
| **Targeted In-Place Editing** | **Tested** | Targets specific operator index (`op_index`) in page content stream to prevent replacing identical strings elsewhere. |
| **Overlay Replacement Fallback** | **Tested** | Draws safe background mask over target bounding box and appends replacement text when in-place modification is constrained. |
| **Custom Font Subsets / Type 0 CMaps** | **Partially Supported** | Text extraction supported via Unicode mapping; direct in-place glyph re-encoding for subset embedded fonts falls back to overlay replacement. |

---

## 4. Document Optimization & Compression

| Capability | Status | Notes |
| :--- | :--- | :--- |
| **Flate Stream Compression** | **Tested** | Re-compresses uncompressed or raw content streams using standard Zlib/Flate. |
| **Object Graph Pruning** | **Tested** | Recursively traverses reachable objects from trailer `/Root` and eliminates unreachable orphan objects. |
| **Image Downsampling** | **Planned** | Lossy recompression requires explicit user quality presets; not performed automatically. |

---

## 5. Security & Redaction

| Capability | Status | Notes |
| :--- | :--- | :--- |
| **Destructive Redaction** | **Unsupported** | Drawing rectangles over text is a visual cover-up, NOT true redaction. Genuine sanitization (permanent vector and text removal) is marked unsupported. |
| **Metadata Sanitization** | **Planned** | Stripping metadata, XML packets, and thumbnails upon export. |

---

## 6. Platform Support Matrix

| Platform | Architecture | Status | Tested Environments |
| :--- | :--- | :--- | :--- |
| **Windows** | `x86_64` | **Tested** | Windows 10, Windows 11 (MinGW GNU & MSVC) |
| **Linux** | `x86_64` | **Tested** | Ubuntu 22.04 LTS, Ubuntu 24.04 LTS (CI) |
| **macOS** | Apple Silicon (`aarch64`) | **Tested** | macOS 14 Sonoma (CI) |
| **macOS** | Intel (`x86_64`) | **Tested** | macOS 13 Ventura (CI) |
