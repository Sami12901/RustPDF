# Changelog

All notable changes to RustPDF are documented here.

## [0.5.0] - Unreleased

### Fixed
- Make page deletion validate every requested index and deduplicate repeated indices before enforcing the last-page invariant.
- Reject duplicate and out-of-range page indices during page reordering.
- Validate split ranges before writing output files.
- Preserve inherited `MediaBox`, `CropBox`, `Resources`, and `Rotate` attributes when page-tree operations normalize nested `/Pages` nodes.
- Add selected-page extraction that respects the caller's requested page order.
- Reject page zero, out-of-range pages, and invalid scale values in CLI rendering.
- Avoid trying to create an empty directory when CLI rendering targets a filename in the current directory.
- Display the actual Cargo package version in the desktop About dialog and CLI version output.
- Report unavailable OCR, redaction, certificate-signing, and file-comparison tools instead of opening the wrong file or silently doing nothing.

### Security
- Refuse to remove a PDF `/Encrypt` dictionary without decrypting encrypted streams and strings. The operation now returns an explicit unsupported error and leaves the document unchanged.
- Display errors from password-protection and remove-protection actions; real PDF Standard Security encryption/decryption is still unsupported.

### Build and release
- Align the workspace package versions at `0.5.0`.
- Require the committed lockfile for CI and release builds.
- Verify release tag version against `Cargo.toml`.
- Include both project license files and per-archive SHA-256 checksums in release assets.

### Regression coverage
- Add tests for nested page-tree inherited attributes, invalid and duplicate page indices, invalid split ranges, extraction order, and CLI render/extract validation.

### Known limitations
- OCR, genuine destructive redaction, PDF password encryption/decryption, digital certificate signing, image downsampling, and visual file comparison are not implemented in this release. See [FEATURES.md](FEATURES.md) and [COMPATIBILITY.md](COMPATIBILITY.md).

## [0.4.0]

See the Git history and tagged release for earlier changes.
