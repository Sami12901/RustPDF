# RustPdf — User Guide

Welcome to **RustPdf**, the high-performance desktop PDF workbench.

---

## 1. Getting Started

### Launching the Application
Launch the application from your terminal or desktop shortcut:
```bash
cargo run -p rustpdf
```

Upon startup, you will be greeted by the **Home Workspace**, which features:
- **Quick Action Cards**: Open Document, New Blank PDF, Combine Files, Split Document, Appearance, and Thumbnails.
- **Recent Documents**: Lists recently opened files with page count, file size, last opened timestamp, and pin status.
- **Drag & Drop**: Simply drag any `.pdf` file from your desktop into the window to open it immediately.

---

## 2. Document Viewer

### View Modes
- **Continuous Scroll**: Vertical layout with smooth scrolling across all pages.
- **Single Page**: Focus on a single page at a time.
- **Two-Page Spread**: Side-by-side spread layout for reading books and reports.

### Zoom & Navigation
- **Zoom In**: <kbd>Ctrl</kbd> + <kbd>+</kbd> (or <kbd>Cmd</kbd> + <kbd>+</kbd>)
- **Zoom Out**: <kbd>Ctrl</kbd> + <kbd>-</kbd> (or <kbd>Cmd</kbd> + <kbd>-</kbd>)
- **Actual Size (100%)**: Reset zoom to 100% scale.
- **Fit Width / Fit Page**: Automatically scale page dimensions to match the active window width or viewport.

---

## 3. Text Selection & Search

- **Selecting Text**: Click and drag your cursor over any text on the page. Press <kbd>Ctrl+C</kbd> to copy the extracted text to your system clipboard.
- **Find in Document**: Press <kbd>Ctrl+F</kbd> to open the search bar.
  - Type your search query to see matches highlighted in real time.
  - Navigate matches using **Next (Enter)** and **Previous (Shift+Enter)**.
  - Toggle **Match Case** (`Aa`) or **Match Whole Words** (`W`).

---

## 4. Page Organization Workspace

Switch to the **Organize Pages** workspace using the toolbar button or by pressing <kbd>Ctrl+K</kbd> and selecting "Organize Pages".

- **Selecting Pages**:
  - Click a thumbnail card to select a single page.
  - <kbd>Ctrl</kbd> + Click to toggle selection of individual pages.
  - <kbd>Shift</kbd> + Click to select a range of contiguous pages.
- **Operations**:
  - **Rotate 90° CW / CCW**: Physically rotates the selected pages in the document.
  - **Delete**: Deletes selected pages (enforces that at least 1 page remains).
  - **Duplicate**: Deep-copies selected pages into the document.
  - **Insert Blank**: Adds a fresh page with standard dimensions.
  - **Undo / Redo**: Press <kbd>Ctrl+Z</kbd> to undo and <kbd>Ctrl+Y</kbd> to redo any page operation.

---

## 5. Combining and Splitting Documents

### Combine Files
1. Open Tools ▸ **Combine Files...**.
2. Click **Add File(s)...** to pick two or more PDF files.
3. Reorder the input list to set the merged page order.
4. Click **Combine Now** to generate the unified document.

### Split Document
1. Open Tools ▸ **Split Document...**.
2. Specify the chunk size (e.g. every 5 pages) or single-page split.
3. Choose the destination folder.
4. Click **Split Now**. Files will be named `{original}_part_001.pdf`, `{original}_part_002.pdf`, etc.

---

## 6. Appearance & Theme Customization

RustPdf provides extensive appearance customization under Settings ▸ **Appearance & Themes**:
- **Theme Modes**: Dark, Light, System.
- **Accent Colors**: Rust Orange, Ocean Blue, Emerald Green, Royal Violet, Nordic Teal, or Custom Accent using the RGB color picker.
- **Theme Portability**: Click **Export JSON** to save your theme, or **Import JSON** to load a custom theme preset. All settings persist automatically across restarts.

---

## 7. Keyboard Shortcuts Summary

| Shortcut | Action |
| :--- | :--- |
| <kbd>Ctrl+O</kbd> | Open PDF file |
| <kbd>Ctrl+S</kbd> | Save document |
| <kbd>Ctrl+Shift+S</kbd> | Save As new file |
| <kbd>Ctrl+F</kbd> | Open Find bar |
| <kbd>Ctrl+K</kbd> | Open Command Palette |
| <kbd>Ctrl+Z</kbd> | Undo last action |
| <kbd>Ctrl+Y</kbd> | Redo undone action |
| <kbd>Ctrl++</kbd> | Zoom In |
| <kbd>Ctrl+-</kbd> | Zoom Out |
| <kbd>Ctrl+0</kbd> | Actual Size (100%) |
| <kbd>Ctrl+W</kbd> | Close active tab |
