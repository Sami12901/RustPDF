//! rustpdf-engine — DocumentSession facade unifying model, rendering, organization, and history.

use rustpdf_cos::CosDoc;
use rustpdf_edit::{save_atomic, EditError, HistoryStack};
use rustpdf_model::{DocumentMetadata, DocumentModel};
use rustpdf_organize::OrganizeError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("cos error: {0}")]
    Cos(#[from] rustpdf_cos::CosError),
    #[error("model error: {0}")]
    Model(#[from] rustpdf_model::ModelError),
    #[error("organize error: {0}")]
    Organize(#[from] OrganizeError),
    #[error("edit error: {0}")]
    Edit(#[from] EditError),
    #[error("annot error: {0}")]
    Annot(#[from] rustpdf_annot::AnnotError),
    #[error("forms error: {0}")]
    Forms(#[from] rustpdf_forms::FormError),
    #[error("security error: {0}")]
    Security(#[from] rustpdf_cos::security::SecurityError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// An active document session representing an open PDF tab.
pub struct DocumentSession {
    pub id: u64,
    pub file_path: Option<PathBuf>,
    pub display_name: String,
    pub is_dirty: bool,
    pub raw_bytes: Arc<Vec<u8>>,
    pub cos_doc: CosDoc,
    pub model: DocumentModel,
    pub history: HistoryStack,
}

impl DocumentSession {
    /// Opens a document from memory buffer with optional file path.
    pub fn open_mem(bytes: Vec<u8>, file_path: Option<PathBuf>) -> Result<Self, EngineError> {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let display_name = file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("Untitled-{id}.pdf"));

        let file_size = bytes.len();
        let cos_doc = CosDoc::load_mem(&bytes)?;
        let model = DocumentModel::from_cos(&cos_doc, file_size)?;

        Ok(Self {
            id,
            file_path,
            display_name,
            is_dirty: false,
            raw_bytes: Arc::new(bytes),
            cos_doc,
            model,
            history: HistoryStack::new(50),
        })
    }

    /// Opens a document directly from a disk path.
    pub fn open_file(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let p = path.as_ref();
        let bytes = std::fs::read(p)?;
        Self::open_mem(bytes, Some(p.to_path_buf()))
    }

    /// Returns the number of pages.
    pub fn page_count(&self) -> usize {
        self.model.pages.len()
    }

    /// Refreshes model and raw_bytes from current in-memory cos_doc.
    fn commit_in_memory_change(
        &mut self,
        description: &str,
        old_bytes: Vec<u8>,
    ) -> Result<(), EngineError> {
        let mut new_bytes = Vec::new();
        self.cos_doc.save_to(&mut new_bytes)?;

        self.history.push(description, old_bytes, new_bytes.clone());
        self.raw_bytes = Arc::new(new_bytes);
        self.model = DocumentModel::from_cos(&self.cos_doc, self.raw_bytes.len())?;
        self.is_dirty = true;
        Ok(())
    }

    /// Rotates specified pages by degrees.
    pub fn rotate_pages(&mut self, pages: &[usize], degrees: i32) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_organize::rotate_pages(self.cos_doc.inner_mut(), pages, degrees)?;
        self.commit_in_memory_change(&format!("Rotate {degrees}°"), old_bytes)
    }

    /// Deletes specified pages.
    pub fn delete_pages(&mut self, pages: &[usize]) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_organize::delete_pages(self.cos_doc.inner_mut(), pages)?;
        self.commit_in_memory_change("Delete pages", old_bytes)
    }

    /// Inserts a blank page.
    pub fn insert_blank_page(
        &mut self,
        index: usize,
        width: f32,
        height: f32,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_organize::insert_blank_page(self.cos_doc.inner_mut(), index, width, height)?;
        self.commit_in_memory_change("Insert blank page", old_bytes)
    }

    /// Duplicates a page.
    pub fn duplicate_page(&mut self, index: usize) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_organize::duplicate_page(self.cos_doc.inner_mut(), index)?;
        self.commit_in_memory_change("Duplicate page", old_bytes)
    }

    /// Moves a page from one index to another (0-indexed) with undo/redo history.
    pub fn move_page(&mut self, from: usize, to: usize) -> Result<(), EngineError> {
        if from == to {
            return Ok(());
        }
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_organize::move_page(self.cos_doc.inner_mut(), from, to)?;
        self.commit_in_memory_change(&format!("Move page {} to {}", from + 1, to + 1), old_bytes)
    }

    /// Reorders pages according to the specified index order with undo/redo history.
    pub fn reorder_pages(&mut self, new_order: &[usize]) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_organize::reorder_pages(self.cos_doc.inner_mut(), new_order)?;
        self.commit_in_memory_change("Reorder pages", old_bytes)
    }

    /// Undoes the last modification.
    pub fn undo(&mut self) -> Result<bool, EngineError> {
        if let Some(restored_bytes) = self.history.undo() {
            self.cos_doc = CosDoc::load_mem(&restored_bytes)?;
            self.model = DocumentModel::from_cos(&self.cos_doc, restored_bytes.len())?;
            self.raw_bytes = Arc::new(restored_bytes);
            self.is_dirty = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redoes the undone modification.
    pub fn redo(&mut self) -> Result<bool, EngineError> {
        if let Some(restored_bytes) = self.history.redo() {
            self.cos_doc = CosDoc::load_mem(&restored_bytes)?;
            self.model = DocumentModel::from_cos(&self.cos_doc, restored_bytes.len())?;
            self.raw_bytes = Arc::new(restored_bytes);
            self.is_dirty = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Saves document atomically to the designated path (or current file_path).
    pub fn save_to(&mut self, target_path: &Path) -> Result<(), EngineError> {
        save_atomic(target_path, &self.raw_bytes)?;
        self.file_path = Some(target_path.to_path_buf());
        self.display_name = target_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or(self.display_name.clone());
        self.is_dirty = false;
        Ok(())
    }

    /// Updates metadata and saves to in-memory tree.
    pub fn update_metadata(&mut self, new_meta: DocumentMetadata) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        new_meta.write_to_cos(&mut self.cos_doc);
        self.commit_in_memory_change("Update metadata", old_bytes)
    }

    /// Adds an annotation and records in undo history.
    pub fn add_annotation(
        &mut self,
        annot: &rustpdf_annot::Annotation,
    ) -> Result<lopdf::ObjectId, EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        let id = rustpdf_annot::add_annotation(self.cos_doc.inner_mut(), annot)?;
        self.commit_in_memory_change("Add annotation", old_bytes)?;
        Ok(id)
    }

    /// Deletes an annotation by ID and records in undo history.
    pub fn delete_annotation(&mut self, annot_id: (u32, u16)) -> Result<bool, EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        let removed = rustpdf_annot::delete_annotation(self.cos_doc.inner_mut(), annot_id)?;
        if removed {
            self.commit_in_memory_change("Delete annotation", old_bytes)?;
        }
        Ok(removed)
    }

    /// Reads annotations on a specific page.
    pub fn get_annotations(&self, page_index: usize) -> Vec<rustpdf_annot::Annotation> {
        rustpdf_annot::get_annotations(self.cos_doc.inner(), page_index)
    }

    /// Reads all interactive form fields.
    pub fn get_form_fields(&self) -> Vec<rustpdf_forms::FormField> {
        rustpdf_forms::get_form_fields(self.cos_doc.inner())
    }

    /// Sets form field value and records in undo history.
    pub fn set_form_field_value(
        &mut self,
        field_name: &str,
        new_val: &str,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_forms::set_field_value(self.cos_doc.inner_mut(), field_name, new_val)?;
        self.commit_in_memory_change("Edit form field", old_bytes)?;
        Ok(())
    }

    /// Resets all form fields and records in undo history.
    pub fn reset_form_fields(&mut self) -> Result<usize, EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        let count = rustpdf_forms::reset_all_fields(self.cos_doc.inner_mut())?;
        self.commit_in_memory_change("Reset form fields", old_bytes)?;
        Ok(count)
    }

    /// Protects the document with user/owner passwords and permissions.
    ///
    /// NOTE: Full ISO 32000 Standard Security encryption is currently unsupported.
    /// Fake pseudo-encryption is prohibited to prevent document corruption.
    pub fn protect_document(
        &mut self,
        user_pass: &str,
        owner_pass: &str,
        perms: rustpdf_cos::security::DocumentPermissions,
    ) -> Result<(), EngineError> {
        rustpdf_cos::security::protect_document(
            self.cos_doc.inner_mut(),
            user_pass,
            owner_pass,
            perms,
        )?;
        Ok(())
    }

    /// Removes password protection only when the security handler can safely decrypt it.
    ///
    /// The current implementation deliberately refuses encrypted PDFs rather than
    /// deleting their /Encrypt dictionary without decrypting strings and streams.
    /// Returns `Ok(false)` when the PDF is not encrypted.
    pub fn remove_protection(&mut self) -> Result<bool, EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        let removed = rustpdf_cos::security::remove_protection(self.cos_doc.inner_mut())?;
        if removed {
            self.commit_in_memory_change("Remove protection", old_bytes)?;
        }
        Ok(removed)
    }

    /// Adds an embedded file attachment to the document.
    pub fn add_attachment(
        &mut self,
        filename: &str,
        data: &[u8],
    ) -> Result<lopdf::ObjectId, EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        let id = rustpdf_model::add_attachment(self.cos_doc.inner_mut(), filename, data)?;
        self.commit_in_memory_change("Add attachment", old_bytes)?;
        Ok(id)
    }

    /// Adds custom text to a page at (x, y) coordinates and commits to undo history.
    #[allow(clippy::too_many_arguments)]
    pub fn add_text_to_page(
        &mut self,
        page_index: usize,
        text: &str,
        x: f32,
        y: f32,
        font: rustpdf_fonts::StandardFont,
        size_pt: f32,
        color_rgb: [f32; 3],
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::add_text_to_page(
            self.cos_doc.inner_mut(),
            page_index,
            text,
            x,
            y,
            font,
            size_pt,
            color_rgb,
        )?;
        self.commit_in_memory_change(&format!("Add text '{text}'"), old_bytes)
    }

    /// Embeds an image (PNG/JPEG) onto a page at (x, y) and commits to undo history.
    pub fn insert_image_to_page(
        &mut self,
        page_index: usize,
        image_bytes: &[u8],
        x: f32,
        y: f32,
        width_pt: f32,
        height_pt: f32,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::insert_image_to_page(
            self.cos_doc.inner_mut(),
            page_index,
            image_bytes,
            x,
            y,
            width_pt,
            height_pt,
        )?;
        self.commit_in_memory_change("Insert image", old_bytes)
    }

    /// Searches and replaces text within a page's content streams and commits to undo history.
    pub fn search_and_replace_text(
        &mut self,
        page_index: usize,
        target: &str,
        replacement: &str,
    ) -> Result<usize, EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        let count = rustpdf_edit::search_and_replace_text_in_page(
            self.cos_doc.inner_mut(),
            page_index,
            target,
            replacement,
        )?;
        if count > 0 {
            self.commit_in_memory_change(
                &format!("Replace '{target}' with '{replacement}'"),
                old_bytes,
            )?;
        }
        Ok(count)
    }

    /// Replaces a targeted text block on a page, attempting exact in-place operator replacement first,
    /// then falling back safely to overlay replacement if exact stream editing cannot be completed.
    pub fn replace_text_block(
        &mut self,
        page_index: usize,
        block: &rustpdf_edit::PageTextBlock,
        new_text: &str,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::replace_text_block(self.cos_doc.inner_mut(), page_index, block, new_text)?;
        self.commit_in_memory_change(
            &format!("Edit text '{}' -> '{}'", block.text, new_text),
            old_bytes,
        )
    }

    /// Replaces a targeted text block using clean overlay replacement with optional background masking.
    pub fn replace_text_block_overlay(
        &mut self,
        page_index: usize,
        block: &rustpdf_edit::PageTextBlock,
        new_text: &str,
        mask_bg: bool,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::replace_text_block_overlay(
            self.cos_doc.inner_mut(),
            page_index,
            block,
            new_text,
            mask_bg,
        )?;
        self.commit_in_memory_change(
            &format!("Overlay text '{}' -> '{}'", block.text, new_text),
            old_bytes,
        )
    }

    /// Adds a diagonal watermark across pages and commits to undo history.
    pub fn add_watermark(
        &mut self,
        pages: &[usize],
        text: &str,
        font_size: f32,
        color_rgb: [f32; 3],
        rotation_deg: f32,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::add_watermark(
            self.cos_doc.inner_mut(),
            pages,
            text,
            font_size,
            color_rgb,
            rotation_deg,
        )?;
        self.commit_in_memory_change(&format!("Add watermark '{text}'"), old_bytes)
    }

    /// Adds header and footer across pages and commits to undo history.
    pub fn add_header_footer(
        &mut self,
        pages: &[usize],
        header_text: Option<&str>,
        footer_text: Option<&str>,
        font_size: f32,
        color_rgb: [f32; 3],
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::add_header_footer(
            self.cos_doc.inner_mut(),
            pages,
            header_text,
            footer_text,
            font_size,
            color_rgb,
        )?;
        self.commit_in_memory_change("Add header and footer", old_bytes)
    }

    /// Adds a solid background color behind page contents and commits to undo history.
    pub fn add_background(
        &mut self,
        pages: &[usize],
        color_rgb: [f32; 3],
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::add_background(self.cos_doc.inner_mut(), pages, color_rgb)?;
        self.commit_in_memory_change("Add background", old_bytes)
    }

    /// Stamps sequential Bates numbering across pages and commits to undo history.
    pub fn add_bates_numbering(
        &mut self,
        pages: &[usize],
        prefix: &str,
        start_num: u32,
        digits: usize,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::add_bates_numbering(
            self.cos_doc.inner_mut(),
            pages,
            prefix,
            start_num,
            digits,
        )?;
        self.commit_in_memory_change("Add Bates numbering", old_bytes)
    }

    /// Adds a clickable hyperlink annotation to a page and commits to undo history.
    pub fn add_link(
        &mut self,
        page_index: usize,
        rect: rustpdf_geom::Rect,
        url: &str,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::add_link(self.cos_doc.inner_mut(), page_index, rect, url)?;
        self.commit_in_memory_change(&format!("Add link to {url}"), old_bytes)
    }

    /// Extracts text blocks with bounding box coordinates for interactive in-page editing.
    pub fn get_page_text_blocks(&self, page_index: usize) -> Vec<rustpdf_edit::PageTextBlock> {
        rustpdf_edit::extract_page_text_blocks(self.cos_doc.inner(), page_index)
    }

    /// Extracts image blocks with bounding box coordinates for interactive in-page editing and replacement.
    pub fn get_page_image_blocks(&self, page_index: usize) -> Vec<rustpdf_edit::PageImageBlock> {
        rustpdf_edit::extract_page_image_blocks(self.cos_doc.inner(), page_index)
    }

    /// Replaces an existing image block on a page with new image data, preserving position and scaling.
    pub fn replace_image_block(
        &mut self,
        page_index: usize,
        block: &rustpdf_edit::PageImageBlock,
        new_image_bytes: &[u8],
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::replace_page_image(
            self.cos_doc.inner_mut(),
            page_index,
            block,
            new_image_bytes,
        )?;
        self.commit_in_memory_change(&format!("Replace image '{}'", block.name), old_bytes)
    }

    /// Deletes an image block from a page and commits to undo history.
    pub fn delete_image_block(
        &mut self,
        page_index: usize,
        block: &rustpdf_edit::PageImageBlock,
    ) -> Result<(), EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        rustpdf_edit::delete_page_image(self.cos_doc.inner_mut(), page_index, block)?;
        self.commit_in_memory_change(&format!("Delete image '{}'", block.name), old_bytes)
    }

    /// Compresses uncompressed streams, prunes orphan objects, and records in undo history.
    pub fn compress_document(&mut self) -> Result<rustpdf_edit::CompressionResult, EngineError> {
        let old_bytes = (*self.raw_bytes).clone();
        let res = rustpdf_edit::compress_document(self.cos_doc.inner_mut())?;
        self.commit_in_memory_change("Compress document", old_bytes)?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let blank_bytes =
            rustpdf_export::create_blank_pdf(2, 612.0, 792.0, "Session Test").unwrap();
        let mut session = DocumentSession::open_mem(blank_bytes, None).unwrap();

        assert_eq!(session.page_count(), 2);
        assert!(!session.is_dirty);

        // Rotate page
        session.rotate_pages(&[0], 90).unwrap();
        assert!(session.is_dirty);
        assert_eq!(session.model.pages[0].rotation, 90);

        // Undo
        assert!(session.undo().unwrap());
        assert_eq!(session.model.pages[0].rotation, 0);

        // Redo
        assert!(session.redo().unwrap());
        assert_eq!(session.model.pages[0].rotation, 90);
    }
}
