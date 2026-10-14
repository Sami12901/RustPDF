# Changelog

All notable changes to RustPDF are documented here.

## [0.5.0] — Release candidate

This version is being prepared on the `release/v0.5.0-audit-hardening` branch. It is not published as a GitHub Release until the release quality gates pass and the release tag is created.

### Correctness and data integrity
- Reject out-of-range page selections and duplicate page indices when reordering PDF pages.
- Deduplicate page-deletion selections and validate that at least one page remains.
- Make bulk form resets all-or-nothing; do not silently ignore reset errors.
- Enforce read-only form field flags when setting or resetting values.
- Validate CLI render scale and page numbers before starting a render.
- Reject out-of-range extraction page numbers instead of silently ignoring them.
- Add regression tests for page indexing, form read-only behavior, atomic saves, and CLI input validation.

### Desktop application behavior
- Require an explicit save, discard, or cancel decision before closing a dirty tab.
- Intercept native application close requests when documents contain unsaved changes.
- Replace immediate process termination with a save-all/discard/cancel exit flow.
- Report failures from common document editing and password-protection operations instead of closing the dialog as if the operation succeeded.
- Persist reset appearance and thumbnail settings and report configuration write failures.
- Use the Cargo package version in CLI version output.

### Build and release
- Upgrade the workspace package version to 0.5.0.
- Keep workspace versions synchronized in `Cargo.lock`.
- Align release documentation with the Rust 1.95 minimum toolchain.
- Add release quality gates so release artifacts are not packaged before validation.

### Limitations that remain
This release candidate does not claim complete PdfCraft feature parity. Standard PDF password encryption, genuine destructive redaction, OCR, image downsampling, digital signatures, and other explicitly planned or unsupported capabilities remain unavailable until implemented and independently tested. See [FEATURES.md](FEATURES.md) and [COMPATIBILITY.md](COMPATIBILITY.md).
