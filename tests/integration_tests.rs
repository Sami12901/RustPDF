//! Cross-crate integration tests for RustPdf

use rustpdf_engine::DocumentSession;
use rustpdf_export::create_blank_pdf;
use rustpdf_organize::{combine_documents, split_document, SplitRule};
use std::fs;

#[test]
fn test_full_document_lifecycle() {
    // 1. Create a 3-page blank document
    let initial_bytes = create_blank_pdf(3, 612.0, 792.0, "Test Document").expect("create blank");
    let mut session = DocumentSession::open_mem(initial_bytes, None).expect("open session");
    assert_eq!(session.page_count(), 3);
    assert!(!session.is_dirty);

    // 2. Rotate page 0 by 90 degrees
    session.rotate_pages(&[0], 90).expect("rotate page");
    assert_eq!(session.model.pages[0].rotation, 90);
    assert!(session.is_dirty);

    // 3. Delete page 2
    session.delete_pages(&[2]).expect("delete page");
    assert_eq!(session.page_count(), 2);

    // 4. Insert a new blank page at index 1
    session.insert_blank_page(1, 595.0, 842.0).expect("insert blank A4");
    assert_eq!(session.page_count(), 3);

    // 5. Test Undo: revert insert blank
    assert!(session.undo().expect("undo"));
    assert_eq!(session.page_count(), 2);

    // 6. Test Redo: re-apply insert blank
    assert!(session.redo().expect("redo"));
    assert_eq!(session.page_count(), 3);

    // 7. Test atomic save and reopen
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let target_path = temp_dir.path().join("saved_lifecycle.pdf");

    session.save_to(&target_path).expect("save atomic");
    assert!(!session.is_dirty);
    assert!(target_path.exists());

    let reopened = DocumentSession::open_file(&target_path).expect("reopen saved file");
    assert_eq!(reopened.page_count(), 3);
}

#[test]
fn test_combine_and_split_pipeline() {
    let doc_a = create_blank_pdf(2, 612.0, 792.0, "Doc A").expect("doc a");
    let doc_b = create_blank_pdf(3, 612.0, 792.0, "Doc B").expect("doc b");

    // Combine 2 + 3 = 5 pages
    let combined_bytes = combine_documents(&[doc_a, doc_b]).expect("combine");
    let combined_session = DocumentSession::open_mem(combined_bytes.clone(), None).expect("open combined");
    assert_eq!(combined_session.page_count(), 5);

    // Split 5 pages every 2 pages -> chunks of [2, 2, 1]
    let chunks = split_document(&combined_bytes, SplitRule::EveryN(2)).expect("split");
    assert_eq!(chunks.len(), 3);

    let chunk_0 = DocumentSession::open_mem(chunks[0].clone(), None).expect("chunk 0");
    let chunk_1 = DocumentSession::open_mem(chunks[1].clone(), None).expect("chunk 1");
    let chunk_2 = DocumentSession::open_mem(chunks[2].clone(), None).expect("chunk 2");

    assert_eq!(chunk_0.page_count(), 2);
    assert_eq!(chunk_1.page_count(), 2);
    assert_eq!(chunk_2.page_count(), 1);
}
