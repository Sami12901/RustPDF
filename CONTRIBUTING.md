# Contributing to RustPdf

Thank you for your interest in contributing to **RustPdf**!

---

## 1. Clean-Room Implementation Rules

To preserve open-source integrity and avoid intellectual property disputes:
- **No Adobe Source Code or Assets**: Never disassemble, reverse-engineer, or copy code, icons, artwork, or fonts from Adobe Acrobat or other proprietary software.
- **Specification-Based**: Implement features strictly based on public standards (ISO 32000-1, ISO 32000-2).
- **No GPL/AGPL Contamination**: RustPdf is dual-licensed MIT OR Apache-2.0. Do not link or vendor GPL/AGPL-licensed libraries.

---

## 2. Development Workflow

1. Fork and clone the repository.
2. Create a feature branch: `git checkout -b feature/my-feature`.
3. Ensure all tests pass: `cargo test --workspace`.
4. Ensure clippy passes: `cargo clippy --workspace -- -D warnings`.
5. Format code: `cargo fmt --all`.
6. Submit a descriptive Pull Request.

---

## 3. Code Conventions

- Keep the core engine (`crates/rustpdf-*`) completely independent of the UI framework (`egui`/`eframe`).
- Follow the transaction command pattern in `rustpdf-edit` for any operations that mutate the document state.
- Ensure all public APIs are documented with doc comments.
