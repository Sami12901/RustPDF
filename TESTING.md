# RustPdf — Testing & Quality Gates

This document describes the testing strategy, test fixtures, and verification procedures for **RustPdf**.

---

## 1. Test Architecture

The test suite is organized into three levels:
1. **Unit Tests**: Embedded within each crate (`src/lib.rs` and modules), testing algorithmic correctness in isolation (geometry, filters, cos helpers, organize functions, theme tokens).
2. **Integration Tests** (`tests/integration_tests.rs`): Cross-crate workflow tests simulating full lifecycle: document creation -> page modifications -> undo/redo -> atomic save -> disk reopen.
3. **CLI End-to-End Tests**: Invoking `rustpdf-cli` commands (`info`, `text`, `render`, `combine`, `split`) and verifying file outputs.

---

## 2. Running Automated Tests

### Workspace Unit Tests
```bash
cargo test --locked --workspace
```

### Integration Tests
```bash
cargo test --test integration_tests
```

### Specific Crate Tests
```bash
cargo test -p rustpdf-geom
cargo test -p rustpdf-filters
cargo test -p rustpdf-cos
cargo test -p rustpdf-organize
cargo test -p rustpdf-edit
cargo test -p rustpdf-annot
cargo test -p rustpdf-forms
```

---

## 3. Key Test Scenarios

- **Geometry**:
  - Rect intersection, union, and containment.
  - Transform matrix multiplication and point projection.
  - Unit conversions (points, inches, millimeters, pixels).
- **Filters**:
  - ASCIIHex encode/decode roundtrip, odd-length trailing digit.
  - RunLength decode of literal runs and repeat runs.
- **Organization**:
  - Rotate pages by ±90°, ±180°, ±270°.
  - Reorder pages and verify ordering invariants.
  - Delete pages and verify failure when attempting to delete the final page.
  - Combine multiple documents and verify total page count.
  - Split document by N pages and verify output part count.
- **Content Editing & Text Replacement (`rustpdf-edit`)**:
  - `test_search_and_replace_text_in_page_tj`: Validates AST operand parsing and text substitution in single-string `Tj` operators.
  - `test_search_and_replace_text_in_page_tj_array`: Validates text replacement inside array-based `TJ` kerning sequences without corrupting kerning integers.
  - `test_search_and_replace_unicode_roundtrip`: Verifies Unicode insertion using UTF-16BE Big-Endian BOM literal encoding.
  - `test_replace_specific_text_block_in_place`: Verifies that targeting a specific `op_index` modifies only the intended block and leaves duplicate text untouched.
  - `test_replace_text_block_overlay`: Validates background-masked text replacement fallback.
- **Document Optimization & Compression (`rustpdf-edit`)**:
  - `test_compress_document_with_uncompressed_stream`: Verifies Flate stream re-compression and unreachable orphan object pruning from trailer `/Root`.
- **Security & Integrity (`rustpdf-cos`)**:
  - `test_protect_document_rejects_pseudo_encryption`: Asserts that pseudo-encryption dictionaries are rejected to prevent false security.
  - `test_remove_protection_refuses_to_drop_encryption_metadata`: Asserts that removing encryption metadata is refused without decryption.
  - Page-operation regressions cover duplicate deletion indices, duplicate reorder indices, invalid split ranges, inherited attributes on nested page trees, and extraction in requested order.
- **Forms & Appearance (`rustpdf-forms`)**:
  - Form editing verifies `/NeedAppearances true` is set on AcroForm dictionary to trigger viewer rendering.
- **Safe Saving**:
  - Atomic save writes to temp file and replaces target atomically.
  - Verification fails if output does not begin with `%PDF` or is corrupted.
  - Bounded undo stack restores document to previous byte-for-byte state.
- **Theme & Thumbnails**:
  - Theme serialization and deserialization from JSON.
  - Thumbnail preset switching (Compact, Balanced, Large).

