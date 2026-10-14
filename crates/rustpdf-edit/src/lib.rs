//! rustpdf-edit — Transaction history, undo/redo stack, and atomic safe saving.

use std::fs;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

pub mod bijoy;
pub use bijoy::*;

pub mod cmap;
pub use cmap::*;

pub mod content;
pub use content::*;

pub mod optimizer;
pub use optimizer::*;

#[derive(Error, Debug)]
pub enum EditError {
    #[error("atomic save failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("output verification failed: written file is invalid")]
    CorruptedSave,
    #[error("lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
    #[error("page index {0} out of bounds")]
    PageOutOfBounds(usize),
    #[error("image error: {0}")]
    ImageError(String),
}

/// A snapshot transaction for deep undo/redo history.
#[derive(Clone, Debug)]
pub struct Transaction {
    pub description: String,
    pub before_bytes: Vec<u8>,
    pub after_bytes: Vec<u8>,
}

/// An undo/redo history stack.
#[derive(Clone, Debug, Default)]
pub struct HistoryStack {
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
    max_depth: usize,
}

impl HistoryStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: max_depth.max(1),
        }
    }

    pub fn push(&mut self, description: impl Into<String>, before: Vec<u8>, after: Vec<u8>) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(Transaction {
            description: description.into(),
            before_bytes: before,
            after_bytes: after,
        });
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|t| t.description.as_str())
    }

    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|t| t.description.as_str())
    }

    pub fn undo(&mut self) -> Option<Vec<u8>> {
        let tx = self.undo_stack.pop()?;
        let restore = tx.before_bytes.clone();
        self.redo_stack.push(tx);
        Some(restore)
    }

    pub fn redo(&mut self) -> Option<Vec<u8>> {
        let tx = self.redo_stack.pop()?;
        let restore = tx.after_bytes.clone();
        self.undo_stack.push(tx);
        Some(restore)
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// Atomically saves bytes to target_path:
/// 1. Creates a temporary file in the same directory.
/// 2. Writes data and flushes buffers to disk (`sync_all`).
/// 3. Validates that the written bytes are a valid PDF.
/// 4. Atomically renames/persists the file over target_path.
pub fn save_atomic(target_path: &Path, bytes: &[u8]) -> Result<(), EditError> {
    // Basic verification: must start with %PDF
    if !bytes.starts_with(b"%PDF") {
        return Err(EditError::CorruptedSave);
    }

    let parent_dir = target_path.parent().unwrap_or(Path::new("."));
    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)?;
    }

    let mut temp_file = tempfile::NamedTempFile::new_in(parent_dir)?;
    temp_file.write_all(bytes)?;
    temp_file.as_file().sync_all()?;

    // Verify it can be loaded with lopdf
    if lopdf::Document::load_mem(bytes).is_err() {
        return Err(EditError::CorruptedSave);
    }

    // Persist replaces the destination atomically on supported OSes
    temp_file.persist(target_path).map_err(|e| e.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_stack() {
        let mut history = HistoryStack::new(10);
        let doc0 = b"version 0".to_vec();
        let doc1 = b"version 1".to_vec();
        let doc2 = b"version 2".to_vec();

        history.push("Edit 1", doc0.clone(), doc1.clone());
        history.push("Edit 2", doc1.clone(), doc2.clone());

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_description(), Some("Edit 2"));

        let restored1 = history.undo().unwrap();
        assert_eq!(restored1, doc1);

        let restored0 = history.undo().unwrap();
        assert_eq!(restored0, doc0);

        assert!(!history.can_undo());
        assert!(history.can_redo());

        let redone1 = history.redo().unwrap();
        assert_eq!(redone1, doc1);
    }

    #[test]
    fn test_save_atomic_replaces_existing_target() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("existing.pdf");

        let mut original_doc = lopdf::Document::with_version("1.7");
        let mut original_bytes = Vec::new();
        original_doc.save_to(&mut original_bytes).unwrap();
        save_atomic(&target, &original_bytes).unwrap();

        let mut replacement_doc = lopdf::Document::with_version("1.4");
        let mut replacement_bytes = Vec::new();
        replacement_doc.save_to(&mut replacement_bytes).unwrap();
        save_atomic(&target, &replacement_bytes).unwrap();

        let written = fs::read(&target).unwrap();
        assert_eq!(written, replacement_bytes);
        assert!(lopdf::Document::load_mem(&written).is_ok());
    }

    #[test]
    fn test_save_atomic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("test_doc.pdf");

        let mut doc = lopdf::Document::with_version("1.7");
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).unwrap();

        save_atomic(&target, &bytes).unwrap();
        assert!(target.exists());

        let read_back = fs::read(&target).unwrap();
        assert_eq!(read_back, bytes);
    }
}
