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
    #[error("render error: {0}")]
    Render(String),
    #[error("image error: {0}")]
    Image(String),
    #[error("text extraction failed on page {page}: {message}")]
    TextExtraction { page: u32, message: String },
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
    info.set(
        b"Title",
        Object::String(title.as_bytes().to_vec(), StringFormat::Literal),
    );
    info.set(
        b"Producer",
        Object::String(b"RustPdf Workbench".to_vec(), StringFormat::Literal),
    );
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
        page.set(
            b"Resources",
            Object::Reference(doc.add_object(Dictionary::new())),
        );

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

/// Extracts text page-by-page and preserves extraction errors with page context.
pub fn try_extract_document_text(doc: &LopdfDoc) -> Result<Vec<String>, ExportError> {
    let page_count = doc.get_pages().len();
    let mut texts = Vec::with_capacity(page_count);
    for idx in 0..page_count {
        let page_num = (idx + 1) as u32;
        let text = doc
            .extract_text(&[page_num])
            .map_err(|error| ExportError::TextExtraction {
                page: page_num,
                message: error.to_string(),
            })?;
        texts.push(text);
    }
    Ok(texts)
}

/// Best-effort text extraction for callers that explicitly accept empty text on failure.
/// UI and file-export workflows should use `try_extract_document_text` instead.
pub fn extract_document_text(doc: &LopdfDoc) -> Vec<String> {
    let page_count = doc.get_pages().len();
    try_extract_document_text(doc).unwrap_or_else(|_| vec![String::new(); page_count])
}

/// Fallible text export. Returns an error instead of producing an incomplete file.
pub fn try_export_plain_text_doc(doc: &LopdfDoc) -> Result<String, ExportError> {
    let pages = try_extract_document_text(doc)?;
    Ok(pages.join("\n\x0C\n"))
}

/// Best-effort compatibility wrapper. Prefer `try_export_plain_text_doc` for user-facing exports.
pub fn export_plain_text_doc(doc: &LopdfDoc) -> String {
    try_export_plain_text_doc(doc).unwrap_or_default()
}

/// Escapes special HTML characters.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Exports document to clean, responsive HTML with styled page containers and layout.
pub fn try_export_html_doc(doc: &LopdfDoc, title: &str) -> Result<String, ExportError> {
    let pages = try_extract_document_text(doc)?;
    let escaped_title = escape_html(title);

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str(&format!("<title>{}</title>\n", escaped_title));
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; background: #f0f2f5; color: #1e293b; margin: 0; padding: 24px; }\n");
    html.push_str(".document-container { max-width: 820px; margin: 0 auto; }\n");
    html.push_str(".page-card { background: #ffffff; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.08); padding: 48px; margin-bottom: 24px; page-break-after: always; }\n");
    html.push_str(".page-header { font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; color: #94a3b8; border-bottom: 1px solid #e2e8f0; padding-bottom: 8px; margin-bottom: 24px; display: flex; justify-content: space-between; }\n");
    html.push_str(".page-body { font-size: 14px; line-height: 1.6; white-space: pre-wrap; word-wrap: break-word; }\n");
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str("<div class=\"document-container\">\n");

    for (i, page_text) in pages.iter().enumerate() {
        let page_num = i + 1;
        html.push_str(&format!(
            "<div class=\"page-card\" id=\"page-{}\">\n",
            page_num
        ));
        html.push_str("<div class=\"page-header\">\n");
        html.push_str(&format!(
            "<span>{}</span><span>Page {}</span>\n",
            escaped_title, page_num
        ));
        html.push_str("</div>\n");
        html.push_str("<div class=\"page-body\">");
        html.push_str(&escape_html(page_text));
        html.push_str("</div>\n");
        html.push_str("</div>\n");
    }

    html.push_str("</div>\n</body>\n</html>");
    Ok(html)
}

pub fn export_html_doc(doc: &LopdfDoc, title: &str) -> String {
    try_export_html_doc(doc, title).unwrap_or_default()
}

/// Escapes special RTF characters.
fn escape_rtf(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '\n' => out.push_str("\\par\n"),
            '\r' => {}
            '\t' => out.push_str("\\tab "),
            _ if c.is_ascii() => out.push(c),
            _ => {
                // RTF uses signed 16-bit UTF-16 code units, not Unicode scalar values.
                // Encoding UTF-16 here also handles non-BMP characters via surrogate pairs.
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units).iter() {
                    out.push_str(&format!("\\u{}?", *unit as i16));
                }
            }
        }
    }
    out
}

/// Exports document to standard Rich Text Format (RTF) compatible with Microsoft Word and WordPad.
pub fn try_export_rtf_doc(doc: &LopdfDoc, title: &str) -> Result<String, ExportError> {
    let pages = try_extract_document_text(doc)?;

    let mut rtf = String::new();
    rtf.push_str("{\\rtf1\\ansi\\deff0\n");
    rtf.push_str(
        "{\\fonttbl{\\f0\\fnil\\fcharset0 Segoe UI;}{\\f1\\froman\\fcharset0 Times New Roman;}}\n",
    );
    rtf.push_str("{\\colortbl ;\\red30\\green41\\blue59;\\red100\\green116\\blue139;}\n");
    rtf.push_str("\\viewkind4\\uc1\\pard\\cf1\\f0\\fs24\n");

    // Title Header
    rtf.push_str(&format!(
        "\\b\\fs32 {}\\b0\\fs24\\par\n\\par\n",
        escape_rtf(title)
    ));

    for (i, page_text) in pages.iter().enumerate() {
        if i > 0 {
            rtf.push_str("\\page\n");
        }
        rtf.push_str(&format!(
            "\\cf2\\fs18 Page {}\\cf1\\fs24\\par\\par\n",
            i + 1
        ));
        rtf.push_str(&escape_rtf(page_text));
        rtf.push_str("\\par\n");
    }

    rtf.push_str("}\n");
    Ok(rtf)
}

pub fn export_rtf_doc(doc: &LopdfDoc, title: &str) -> String {
    try_export_rtf_doc(doc, title).unwrap_or_default()
}

/// Fallible Word-compatible document export (RTF stream).
pub fn try_export_word_doc(doc: &LopdfDoc, title: &str) -> Result<String, ExportError> {
    try_export_rtf_doc(doc, title)
}

/// Best-effort compatibility wrapper. Prefer `try_export_word_doc` for user-facing exports.
pub fn export_word_doc(doc: &LopdfDoc, title: &str) -> String {
    try_export_word_doc(doc, title).unwrap_or_default()
}

/// Exports extracted text into a CSV spreadsheet format.
pub fn try_export_csv_doc(doc: &LopdfDoc) -> Result<String, ExportError> {
    let pages = try_extract_document_text(doc)?;
    let mut csv = String::new();
    csv.push_str("Page,Line,Text\n");

    for (page_idx, text) in pages.iter().enumerate() {
        let page_num = page_idx + 1;
        for (line_idx, line) in text.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // CSV quote escaping
            let escaped = trimmed.replace('"', "\"\"");
            csv.push_str(&format!("{},{},\"{}\"\n", page_num, line_num, escaped));
        }
    }

    Ok(csv)
}

pub fn export_csv_doc(doc: &LopdfDoc) -> String {
    try_export_csv_doc(doc).unwrap_or_default()
}

/// Synchronously rasterizes a page from PDF bytes and encodes to PNG bytes.
pub fn export_page_png(
    pdf_bytes: &[u8],
    page_index: usize,
    scale: f32,
) -> Result<Vec<u8>, ExportError> {
    let (w, h, rgba) = rustpdf_render::render_page_sync(pdf_bytes, page_index, scale)
        .map_err(ExportError::Render)?;

    let img = image::RgbaImage::from_raw(w, h, rgba)
        .ok_or_else(|| ExportError::Image("Failed to create RGBA image from buffer".to_string()))?;

    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| ExportError::Image(e.to_string()))?;

    Ok(png_bytes)
}

/// Rasterizes all pages of a PDF document and saves them as PNG files in out_dir.
pub fn export_all_pages_png(
    pdf_bytes: &[u8],
    out_dir: &std::path::Path,
    prefix: &str,
    scale: f32,
) -> Result<Vec<std::path::PathBuf>, ExportError> {
    let doc = LopdfDoc::load_mem(pdf_bytes)?;
    let page_count = doc.get_pages().len();
    let mut saved_paths = Vec::with_capacity(page_count);

    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)?;
    }

    for idx in 0..page_count {
        let page_png = export_page_png(pdf_bytes, idx, scale)?;
        let filename = format!("{}_page_{:03}.png", prefix, idx + 1);
        let path = out_dir.join(filename);
        std::fs::write(&path, page_png)?;
        saved_paths.push(path);
    }

    Ok(saved_paths)
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

    #[test]
    fn test_export_formats() {
        let bytes = create_blank_pdf(2, 612.0, 792.0, "Multi-Format Test").unwrap();
        let doc = LopdfDoc::load_mem(&bytes).unwrap();

        let html = export_html_doc(&doc, "Multi-Format Test");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Multi-Format Test"));
        assert!(html.contains("Page 1"));

        let rtf = export_rtf_doc(&doc, "Multi-Format Test");
        assert!(rtf.starts_with("{\\rtf1"));
        assert!(rtf.contains("Multi-Format Test"));

        let csv = export_csv_doc(&doc);
        assert!(csv.starts_with("Page,Line,Text"));
    }

    #[test]
    fn test_fallible_export_formats_return_complete_output() {
        let bytes = create_blank_pdf(2, 612.0, 792.0, "Fallible Export Test").unwrap();
        let doc = LopdfDoc::load_mem(&bytes).unwrap();

        assert_eq!(try_extract_document_text(&doc).unwrap().len(), 2);
        assert!(try_export_plain_text_doc(&doc).is_ok());
        assert!(try_export_html_doc(&doc, "Fallible Export Test")
            .unwrap()
            .contains("</html>"));
        assert!(try_export_rtf_doc(&doc, "Fallible Export Test")
            .unwrap()
            .starts_with("{\\rtf1"));
        assert!(try_export_word_doc(&doc, "Fallible Export Test")
            .unwrap()
            .starts_with("{\\rtf1"));
        assert!(try_export_csv_doc(&doc)
            .unwrap()
            .starts_with("Page,Line,Text"));
    }

    #[test]
    fn test_rtf_encodes_non_bmp_unicode_as_utf16_surrogate_pair() {
        assert_eq!(escape_rtf("😀"), "\\u-10179?\\u-8704?");
        assert_eq!(escape_rtf("tab\tvalue"), "tab\\tab value");
    }

    #[test]
    fn test_export_page_png() {
        let bytes = create_blank_pdf(1, 612.0, 792.0, "Raster Test").unwrap();
        let png = export_page_png(&bytes, 0, 1.0).unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
}
