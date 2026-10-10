use lopdf::Document;
use rustpdf_export::create_blank_pdf;
use rustpdf_organize::{combine_documents, split_document, SplitRule};

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
