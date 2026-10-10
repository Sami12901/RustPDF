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
cargo test --workspace
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
- **Safe Saving**:
  - Atomic save writes to temp file and replaces target atomically.
  - Verification fails if output does not begin with `%PDF` or is corrupted.
  - Undo stack restores document to previous byte-for-byte state.
- **Theme & Thumbnails**:
  - Theme serialization and deserialization from JSON.
  - Thumbnail preset switching (Compact, Balanced, Large).
