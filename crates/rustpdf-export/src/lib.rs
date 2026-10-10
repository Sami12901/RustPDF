//! rustpdf-export — Document generation, text extraction, and JSON metadata export.

use lopdf::{Dictionary, Document as LopdfDoc, Object, Stream, StringFormat};
use rustpdf_model::DocumentModel;
use rustpdf_render::PageText;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Creates a blank PDF document with the specified number of pages, dimensions, and title.
pub fn create_blank_pdf(
    num_pages: usize,
    width_pt: f32,
    height_pt: f32,
    title: &str,
) -> Result<Vec<u8>, ExportError> {
    let mut doc = LopdfDoc::with_version("1.7");

    // Add info dict
    let mut info = Dictionary::new();
    info.set(b"Title", Object::String(title.as_bytes().to_vec(), StringFormat::Literal));
    info.set(b"Producer", Object::String(b"RustPdf Workbench".to_vec(), StringFormat::Literal));
    let info_id = doc.add_object(info);
    doc.trailer.set(b"Info", Object::Reference(info_id));

    // Allocate Pages root object ID
    let pages_root_id = doc.new_object_id();

    // Add Catalog pointing to Pages root
    let mut catalog = Dictionary::new();
    catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
    catalog.set(b"Pages", Object::Reference(pages_root_id));
    let catalog_id = doc.add_object(catalog);
    doc.trailer.set(b"Root", Object::Reference(catalog_id));

    let pages_to_create = num_pages.max(1);
    let mut kid_refs = Vec::with_capacity(pages_to_create);

    for _ in 0..pages_to_create {
        let content = Stream::new(Dictionary::new(), Vec::new());
        let content_id = doc.add_object(content);

        let mut page = Dictionary::new();
        page.set(b"Type", Object::Name(b"Page".to_vec()));
        page.set(b"Parent", Object::Reference(pages_root_id));
        page.set(
            b"MediaBox",
            Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(width_pt),
                Object::Real(height_pt),
            ]),
        );
        page.set(b"Contents", Object::Reference(content_id));
        page.set(b"Resources", Object::Reference(doc.add_object(Dictionary::new())));

        let page_id = doc.add_object(page);
        kid_refs.push(Object::Reference(page_id));
    }

    let mut pages_dict = Dictionary::new();
    pages_dict.set(b"Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set(b"Count", Object::Integer(pages_to_create as i64));
    pages_dict.set(b"Kids", Object::Array(kid_refs));
    doc.set_object(pages_root_id, pages_dict);

    let mut out = Vec::new();
    doc.save_to(&mut out)?;
    Ok(out)
}

/// Combines plain text from multiple pages with form-feed page separators.
pub fn export_plain_text(pages: &[PageText]) -> String {
    let mut out = String::new();
    for (i, p) in pages.iter().enumerate() {
        if i > 0 {
            out.push_str("\n\x0C\n"); // Standard page break Form Feed
        }
        out.push_str(&p.plain_text());
    }
    out
}

/// Exports document model as formatted JSON.
pub fn export_model_json(model: &DocumentModel) -> Result<String, ExportError> {
    let json = serde_json::to_string_pretty(model)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_blank_pdf() {
        let bytes = create_blank_pdf(3, 612.0, 792.0, "Blank Test").unwrap();
        assert!(bytes.starts_with(b"%PDF-1.7"));
        let parsed = LopdfDoc::load_mem(&bytes).unwrap();
        assert_eq!(parsed.get_pages().len(), 3);
    }
}
