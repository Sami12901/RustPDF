use lopdf::Document;
use rustpdf_export::create_blank_pdf;
use rustpdf_organize::{combine_documents, split_document, SplitRule};
use std::process::Command;

#[test]
fn test_cli_workflows_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let pdf1_path = dir.path().join("doc1.pdf");
    let pdf2_path = dir.path().join("doc2.pdf");
    let merged_path = dir.path().join("merged.pdf");
    let split_dir = dir.path().join("split_out");

    let bytes1 = create_blank_pdf(2, 612.0, 792.0, "Doc 1").unwrap();
    let bytes2 = create_blank_pdf(3, 595.0, 842.0, "Doc 2").unwrap();

    std::fs::write(&pdf1_path, &bytes1).unwrap();
    std::fs::write(&pdf2_path, &bytes2).unwrap();

    // 1. Combine
    let combined_bytes = combine_documents(&[bytes1.clone(), bytes2.clone()]).unwrap();
    std::fs::write(&merged_path, &combined_bytes).unwrap();

    let merged_doc = Document::load_mem(&combined_bytes).unwrap();
    assert_eq!(merged_doc.get_pages().len(), 5);

    // 2. Split
    let chunks = split_document(&combined_bytes, SplitRule::EveryN(2)).unwrap();
    assert_eq!(chunks.len(), 3); // 2 pages, 2 pages, 1 page

    std::fs::create_dir_all(&split_dir).unwrap();
    for (i, chunk) in chunks.iter().enumerate() {
        let p = split_dir.join(format!("chunk_{}.pdf", i + 1));
        std::fs::write(&p, chunk).unwrap();
        let loaded = Document::load_mem(chunk).unwrap();
        if i == 2 {
            assert_eq!(loaded.get_pages().len(), 1);
        } else {
            assert_eq!(loaded.get_pages().len(), 2);
        }
    }
}

#[test]
fn test_text_command_extracts_all_pages_and_selected_page() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("text-fixture.pdf");

    let bytes = create_blank_pdf(2, 612.0, 792.0, "Text Extraction Fixture").unwrap();
    let mut pdf = Document::load_mem(&bytes).unwrap();
    rustpdf_edit::add_text_to_page(
        &mut pdf,
        0,
        "FIRST_PAGE_MARKER",
        72.0,
        700.0,
        rustpdf_fonts::StandardFont::Helvetica,
        12.0,
        [0.0, 0.0, 0.0],
    )
    .unwrap();
    rustpdf_edit::add_text_to_page(
        &mut pdf,
        1,
        "SECOND_PAGE_MARKER",
        72.0,
        700.0,
        rustpdf_fonts::StandardFont::Helvetica,
        12.0,
        [0.0, 0.0, 0.0],
    )
    .unwrap();

    let mut encoded = Vec::new();
    pdf.save_to(&mut encoded).unwrap();
    std::fs::write(&input, encoded).unwrap();

    let all_pages = Command::new(env!("CARGO_BIN_EXE_rustpdf-cli"))
        .args(["text", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        all_pages.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&all_pages.stderr)
    );
    let all_stdout = String::from_utf8_lossy(&all_pages.stdout);
    assert!(all_stdout.contains("FIRST_PAGE_MARKER"), "{all_stdout}");
    assert!(all_stdout.contains("SECOND_PAGE_MARKER"), "{all_stdout}");

    let selected_page = Command::new(env!("CARGO_BIN_EXE_rustpdf-cli"))
        .args(["text", input.to_str().unwrap(), "--page", "2"])
        .output()
        .unwrap();
    assert!(
        selected_page.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&selected_page.stderr)
    );
    let selected_stdout = String::from_utf8_lossy(&selected_page.stdout);
    assert!(
        selected_stdout.contains("SECOND_PAGE_MARKER"),
        "{selected_stdout}"
    );
    assert!(
        !selected_stdout.contains("FIRST_PAGE_MARKER"),
        "{selected_stdout}"
    );
}

#[test]
fn test_text_command_rejects_page_zero() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("one-page.pdf");
    let bytes = create_blank_pdf(1, 612.0, 792.0, "Page Validation").unwrap();
    std::fs::write(&input, bytes).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rustpdf-cli"))
        .args(["text", input.to_str().unwrap(), "--page", "0"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("out of range"));

}

#[test]
fn test_extract_cli_preserves_requested_page_order() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("ordered-input.pdf");
    let output = dir.path().join("ordered-output.pdf");

    let mut pdf = Document::with_version("1.7");
    rustpdf_organize::insert_blank_page(&mut pdf, 0, 100.0, 200.0).unwrap();
    rustpdf_organize::insert_blank_page(&mut pdf, 1, 200.0, 300.0).unwrap();
    rustpdf_organize::insert_blank_page(&mut pdf, 2, 300.0, 400.0).unwrap();
    let mut bytes = Vec::new();
    pdf.save_to(&mut bytes).unwrap();
    std::fs::write(&input, bytes).unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_rustpdf-cli"))
        .args([
            "extract",
            input.to_str().unwrap(),
            "--pages",
            "3,1",
            "--out",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let extracted = Document::load(&output).unwrap();
    let pages = extracted.get_pages();
    assert_eq!(pages.len(), 2);
    let first_box = extracted
        .get_object(pages[&1])
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap();
    let second_box = extracted
        .get_object(pages[&2])
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(
        first_box[2]
            .as_float()
            .unwrap_or_else(|_| first_box[2].as_i64().unwrap() as f32),
        300.0
    );
    assert_eq!(
        second_box[2]
            .as_float()
            .unwrap_or_else(|_| second_box[2].as_i64().unwrap() as f32),
        100.0
    );
}

#[test]
fn test_render_cli_rejects_page_zero() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("one-page.pdf");
    let output = dir.path().join("page.png");
    let bytes = create_blank_pdf(1, 612.0, 792.0, "Render Validation").unwrap();
    std::fs::write(&input, bytes).unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_rustpdf-cli"))
        .args([
            "render",
            input.to_str().unwrap(),
            "--page",
            "0",
            "--out",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Page numbers start at 1"));
    assert!(!output.exists());
}
