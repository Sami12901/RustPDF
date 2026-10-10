//! rustpdf-edit/content — PDF content manipulation: text editing/insertion, image embedding, watermarks, headers & footers.

use crate::EditError;
use lopdf::{Dictionary, Document as LopdfDoc, Object, ObjectId, Stream};
use rustpdf_fonts::StandardFont;
use rustpdf_geom::Rect;

/// Escapes a UTF-8 string for safe literal encoding in PDF parentheses `(...)`.
pub fn escape_pdf_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            '\r' => out.push_str("\\r"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Ensures the given standard font is registered in the page's `/Resources /Font` dictionary and returns its alias.
pub fn ensure_page_font(
    doc: &mut LopdfDoc,
    page_id: ObjectId,
    font: StandardFont,
) -> Result<String, EditError> {
    let font_name = font.postscript_name();
    let alias = match font {
        StandardFont::Helvetica => "F_Helv",
        StandardFont::HelveticaBold => "F_HelvB",
        StandardFont::TimesRoman => "F_Times",
        StandardFont::Courier => "F_Cour",
        _ => "F_Std",
    };

    let mut font_dict = Dictionary::new();
    font_dict.set(b"Type", Object::Name(b"Font".to_vec()));
    font_dict.set(b"Subtype", Object::Name(b"Type1".to_vec()));
    font_dict.set(b"BaseFont", Object::Name(font_name.as_bytes().to_vec()));
    let font_obj_id = doc.add_object(font_dict);

    let page = doc
        .get_object_mut(page_id)
        .map_err(EditError::Lopdf)?
        .as_dict_mut()
        .map_err(EditError::Lopdf)?;
    if !page.has(b"Resources") {
        page.set(b"Resources", Object::Dictionary(Dictionary::new()));
    }

    let res_dict = if let Ok(res_ref) = page.get(b"Resources").and_then(Object::as_reference) {
        doc.get_object_mut(res_ref)
            .map_err(EditError::Lopdf)?
            .as_dict_mut()
            .map_err(EditError::Lopdf)?
    } else {
        page.get_mut(b"Resources")
            .map_err(EditError::Lopdf)?
            .as_dict_mut()
            .map_err(EditError::Lopdf)?
    };

    if !res_dict.has(b"Font") {
        res_dict.set(b"Font", Object::Dictionary(Dictionary::new()));
    }

    let fonts = res_dict
        .get_mut(b"Font")
        .map_err(EditError::Lopdf)?
        .as_dict_mut()
        .map_err(EditError::Lopdf)?;
    fonts.set(alias.as_bytes(), Object::Reference(font_obj_id));

    Ok(alias.to_string())
}

/// Appends a stream of PDF content operators to the page's `/Contents` stream.
pub fn append_page_content(
    doc: &mut LopdfDoc,
    page_id: ObjectId,
    content_bytes: Vec<u8>,
) -> Result<(), EditError> {
    let stream = Stream::new(Dictionary::new(), content_bytes);
    let stream_id = doc.add_object(stream);

    let page = doc
        .get_object_mut(page_id)
        .map_err(EditError::Lopdf)?
        .as_dict_mut()
        .map_err(EditError::Lopdf)?;
    match page.get_mut(b"Contents") {
        Ok(Object::Reference(existing_id)) => {
            let arr = vec![
                Object::Reference(*existing_id),
                Object::Reference(stream_id),
            ];
            page.set(b"Contents", Object::Array(arr));
        }
        Ok(Object::Array(ref mut arr)) => {
            arr.push(Object::Reference(stream_id));
        }
        _ => {
            page.set(b"Contents", Object::Reference(stream_id));
        }
    }
    Ok(())
}

/// Prepends a stream of PDF content operators to the page's `/Contents` (e.g. for background).
pub fn prepend_page_content(
    doc: &mut LopdfDoc,
    page_id: ObjectId,
    content_bytes: Vec<u8>,
) -> Result<(), EditError> {
    let stream = Stream::new(Dictionary::new(), content_bytes);
    let stream_id = doc.add_object(stream);

    let page = doc
        .get_object_mut(page_id)
        .map_err(EditError::Lopdf)?
        .as_dict_mut()
        .map_err(EditError::Lopdf)?;
    match page.get_mut(b"Contents") {
        Ok(Object::Reference(existing_id)) => {
            let arr = vec![
                Object::Reference(stream_id),
                Object::Reference(*existing_id),
            ];
            page.set(b"Contents", Object::Array(arr));
        }
        Ok(Object::Array(ref mut arr)) => {
            arr.insert(0, Object::Reference(stream_id));
        }
        _ => {
            page.set(b"Contents", Object::Reference(stream_id));
        }
    }
    Ok(())
}

/// Adds text onto a specified page at (x, y) coordinates.
#[allow(clippy::too_many_arguments)]
pub fn add_text_to_page(
    doc: &mut LopdfDoc,
    page_index: usize,
    text: &str,
    x: f32,
    y: f32,
    font: StandardFont,
    size_pt: f32,
    color_rgb: [f32; 3],
) -> Result<(), EditError> {
    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let &page_id = page_map
        .get(&page_num)
        .ok_or(EditError::PageOutOfBounds(page_index))?;

    let font_alias = ensure_page_font(doc, page_id, font)?;
    let escaped = escape_pdf_string(text);

    let stream_str = format!(
        "q\nBT\n/{} {} Tf\n{:.3} {:.3} {:.3} rg\n1 0 0 1 {:.2} {:.2} Tm\n({}) Tj\nET\nQ\n",
        font_alias, size_pt, color_rgb[0], color_rgb[1], color_rgb[2], x, y, escaped
    );

    append_page_content(doc, page_id, stream_str.into_bytes())
}

/// Embeds an image (PNG, JPEG, etc.) onto a page at (x, y) with (width_pt, height_pt) dimensions.
pub fn insert_image_to_page(
    doc: &mut LopdfDoc,
    page_index: usize,
    image_bytes: &[u8],
    x: f32,
    y: f32,
    width_pt: f32,
    height_pt: f32,
) -> Result<(), EditError> {
    let img =
        image::load_from_memory(image_bytes).map_err(|e| EditError::ImageError(e.to_string()))?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let raw_data = rgb.into_raw();

    let mut img_dict = Dictionary::new();
    img_dict.set(b"Type", Object::Name(b"XObject".to_vec()));
    img_dict.set(b"Subtype", Object::Name(b"Image".to_vec()));
    img_dict.set(b"Width", Object::Integer(w as i64));
    img_dict.set(b"Height", Object::Integer(h as i64));
    img_dict.set(b"ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    img_dict.set(b"BitsPerComponent", Object::Integer(8));

    let stream = Stream::new(img_dict, raw_data);
    let img_id = doc.add_object(stream);

    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let &page_id = page_map
        .get(&page_num)
        .ok_or(EditError::PageOutOfBounds(page_index))?;

    let img_alias = format!("Im{}", img_id.0);

    let page = doc
        .get_object_mut(page_id)
        .map_err(EditError::Lopdf)?
        .as_dict_mut()
        .map_err(EditError::Lopdf)?;
    if !page.has(b"Resources") {
        page.set(b"Resources", Object::Dictionary(Dictionary::new()));
    }
    let res_dict = if let Ok(res_ref) = page.get(b"Resources").and_then(Object::as_reference) {
        doc.get_object_mut(res_ref)
            .map_err(EditError::Lopdf)?
            .as_dict_mut()
            .map_err(EditError::Lopdf)?
    } else {
        page.get_mut(b"Resources")
            .map_err(EditError::Lopdf)?
            .as_dict_mut()
            .map_err(EditError::Lopdf)?
    };

    if !res_dict.has(b"XObject") {
        res_dict.set(b"XObject", Object::Dictionary(Dictionary::new()));
    }
    let xobjs = res_dict
        .get_mut(b"XObject")
        .map_err(EditError::Lopdf)?
        .as_dict_mut()
        .map_err(EditError::Lopdf)?;
    xobjs.set(img_alias.as_bytes(), Object::Reference(img_id));

    let stream_str = format!(
        "q\n{:.2} 0 0 {:.2} {:.2} {:.2} cm\n/{} Do\nQ\n",
        width_pt, height_pt, x, y, img_alias
    );

    append_page_content(doc, page_id, stream_str.into_bytes())
}

/// Encodes a UTF-8 string into PDF literal bytes, using UTF-16BE with BOM for non-ASCII.
pub fn encode_pdf_string_bytes(s: &str) -> Vec<u8> {
    if s.is_ascii() {
        s.as_bytes().to_vec()
    } else {
        let mut bytes = vec![0xFE, 0xFF];
        for u in s.encode_utf16() {
            bytes.extend_from_slice(&u.to_be_bytes());
        }
        bytes
    }
}

/// Searches and replaces literal string occurrences inside the parsed content stream operators of a page.
/// Parses operations with lopdf::content::Content — never performs unsafe raw byte splicing.
pub fn search_and_replace_text_in_page(
    doc: &mut LopdfDoc,
    page_index: usize,
    target: &str,
    replacement: &str,
) -> Result<usize, EditError> {
    if target.is_empty() {
        return Ok(0);
    }
    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let &page_id = page_map
        .get(&page_num)
        .ok_or(EditError::PageOutOfBounds(page_index))?;

    let content_ids: Vec<ObjectId> = {
        let page = doc
            .get_object(page_id)
            .map_err(EditError::Lopdf)?
            .as_dict()
            .map_err(EditError::Lopdf)?;
        match page.get(b"Contents") {
            Ok(Object::Reference(id)) => vec![*id],
            Ok(Object::Array(arr)) => arr.iter().filter_map(|o| o.as_reference().ok()).collect(),
            _ => Vec::new(),
        }
    };

    let mut replacements_count = 0;

    for cid in content_ids {
        if let Ok(Object::Stream(ref mut stream)) = doc.get_object_mut(cid) {
            let data = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());

            let mut content = match lopdf::content::Content::decode(&data) {
                Ok(c) => c,
                Err(_) => continue, // Cannot safely parse this stream, skip safely
            };

            let mut stream_modified = false;
            for op in &mut content.operations {
                match op.operator.as_str() {
                    "Tj" | "'" => {
                        if let Some(Object::String(bytes, fmt)) = op.operands.first_mut() {
                            let text = decode_pdf_bytes(bytes);
                            if text.contains(target) {
                                let count = text.matches(target).count();
                                replacements_count += count;
                                let new_text = text.replace(target, replacement);
                                *bytes = encode_pdf_string_bytes(&new_text);
                                *fmt = lopdf::StringFormat::Literal;
                                stream_modified = true;
                            }
                        }
                    }
                    "\"" => {
                        if let Some(Object::String(bytes, fmt)) = op.operands.get_mut(2) {
                            let text = decode_pdf_bytes(bytes);
                            if text.contains(target) {
                                let count = text.matches(target).count();
                                replacements_count += count;
                                let new_text = text.replace(target, replacement);
                                *bytes = encode_pdf_string_bytes(&new_text);
                                *fmt = lopdf::StringFormat::Literal;
                                stream_modified = true;
                            }
                        }
                    }
                    "TJ" => {
                        if let Some(Object::Array(arr)) = op.operands.first_mut() {
                            let mut array_modified = false;
                            for item in arr.iter_mut() {
                                if let Object::String(bytes, fmt) = item {
                                    let text = decode_pdf_bytes(bytes);
                                    if text.contains(target) {
                                        let count = text.matches(target).count();
                                        replacements_count += count;
                                        let new_text = text.replace(target, replacement);
                                        *bytes = encode_pdf_string_bytes(&new_text);
                                        *fmt = lopdf::StringFormat::Literal;
                                        array_modified = true;
                                    }
                                }
                            }
                            if array_modified {
                                stream_modified = true;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if stream_modified {
                if let Ok(encoded) = content.encode() {
                    stream.set_plain_content(encoded);
                    stream.dict.remove(b"Filter");
                }
            }
        }
    }

    Ok(replacements_count)
}

/// Helper to extract an f32 from an Object (Real or Integer).
fn get_f32_from_obj(obj: &Object) -> Option<f32> {
    match obj {
        Object::Real(f) => Some(*f),
        Object::Integer(i) => Some(*i as f32),
        _ => None,
    }
}

/// Helper to get page dimensions (width, height) in points from MediaBox.
pub fn get_page_dimensions(doc: &LopdfDoc, page_id: ObjectId) -> (f32, f32) {
    if let Ok(page_dict) = doc.get_object(page_id).and_then(Object::as_dict) {
        if let Ok(mb) = page_dict.get(b"MediaBox").and_then(Object::as_array) {
            let pw = mb.get(2).and_then(get_f32_from_obj).unwrap_or(595.0);
            let ph = mb.get(3).and_then(get_f32_from_obj).unwrap_or(842.0);
            return (pw, ph);
        }
    }
    (595.0, 842.0)
}

/// Stretches/places a diagonal text watermark across specified pages.
pub fn add_watermark(
    doc: &mut LopdfDoc,
    pages: &[usize],
    text: &str,
    font_size: f32,
    color_rgb: [f32; 3],
    rotation_deg: f32,
) -> Result<(), EditError> {
    let rad = rotation_deg.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();
    let escaped = escape_pdf_string(text);

    let page_map = doc.get_pages();
    for &page_idx in pages {
        let page_num = (page_idx + 1) as u32;
        if let Some(&page_id) = page_map.get(&page_num) {
            let font_alias = ensure_page_font(doc, page_id, StandardFont::HelveticaBold)?;
            let (w, h) = get_page_dimensions(doc, page_id);

            let cx = w * 0.5;
            let cy = h * 0.5;

            let stream_str = format!(
                "q\nBT\n/{} {} Tf\n{:.3} {:.3} {:.3} rg\n{:.4} {:.4} {:.4} {:.4} {:.2} {:.2} Tm\n({}) Tj\nET\nQ\n",
                font_alias, font_size, color_rgb[0], color_rgb[1], color_rgb[2],
                cos_a, sin_a, -sin_a, cos_a, cx - 120.0, cy, escaped
            );

            let _ = append_page_content(doc, page_id, stream_str.into_bytes());
        }
    }
    Ok(())
}

/// Adds header and footer lines (supporting `{n}` and `{total}` variables) across specified pages.
pub fn add_header_footer(
    doc: &mut LopdfDoc,
    pages: &[usize],
    header_text: Option<&str>,
    footer_text: Option<&str>,
    font_size: f32,
    color_rgb: [f32; 3],
) -> Result<(), EditError> {
    let page_map = doc.get_pages();
    let total_pages = page_map.len();

    for &page_idx in pages {
        let page_num = (page_idx + 1) as u32;
        if let Some(&page_id) = page_map.get(&page_num) {
            let font_alias = ensure_page_font(doc, page_id, StandardFont::Helvetica)?;
            let (_w, h) = get_page_dimensions(doc, page_id);

            let mut stream_str = String::new();
            if let Some(hdr) = header_text {
                let h_rendered = hdr
                    .replace("{n}", &(page_idx + 1).to_string())
                    .replace("{total}", &total_pages.to_string());
                let esc = escape_pdf_string(&h_rendered);
                stream_str.push_str(&format!(
                    "q\nBT\n/{} {} Tf\n{:.3} {:.3} {:.3} rg\n1 0 0 1 36.0 {:.2} Tm\n({}) Tj\nET\nQ\n",
                    font_alias, font_size, color_rgb[0], color_rgb[1], color_rgb[2], h - 28.0, esc
                ));
            }

            if let Some(ftr) = footer_text {
                let f_rendered = ftr
                    .replace("{n}", &(page_idx + 1).to_string())
                    .replace("{total}", &total_pages.to_string());
                let esc = escape_pdf_string(&f_rendered);
                stream_str.push_str(&format!(
                    "q\nBT\n/{} {} Tf\n{:.3} {:.3} {:.3} rg\n1 0 0 1 36.0 20.0 Tm\n({}) Tj\nET\nQ\n",
                    font_alias, font_size, color_rgb[0], color_rgb[1], color_rgb[2], esc
                ));
            }

            if !stream_str.is_empty() {
                let _ = append_page_content(doc, page_id, stream_str.into_bytes());
            }
        }
    }
    Ok(())
}

/// Stamps a colored background rectangle behind all page content.
pub fn add_background(
    doc: &mut LopdfDoc,
    pages: &[usize],
    color_rgb: [f32; 3],
) -> Result<(), EditError> {
    let page_map = doc.get_pages();
    for &page_idx in pages {
        let page_num = (page_idx + 1) as u32;
        if let Some(&page_id) = page_map.get(&page_num) {
            let (w, h) = get_page_dimensions(doc, page_id);

            let stream_str = format!(
                "q\n{:.3} {:.3} {:.3} rg\n0 0 {:.2} {:.2} re\nf\nQ\n",
                color_rgb[0], color_rgb[1], color_rgb[2], w, h
            );

            let _ = prepend_page_content(doc, page_id, stream_str.into_bytes());
        }
    }
    Ok(())
}

/// Applies sequential Bates numbering (e.g. `DOC-000001`) across specified pages.
pub fn add_bates_numbering(
    doc: &mut LopdfDoc,
    pages: &[usize],
    prefix: &str,
    start_num: u32,
    digits: usize,
) -> Result<(), EditError> {
    let page_map = doc.get_pages();
    for (i, &page_idx) in pages.iter().enumerate() {
        let num = start_num + (i as u32);
        let bates_str = format!("{}{:0width$}", prefix, num, width = digits);
        let page_num = (page_idx + 1) as u32;
        if let Some(&page_id) = page_map.get(&page_num) {
            let font_alias = ensure_page_font(doc, page_id, StandardFont::HelveticaBold)?;
            let (w, _h) = get_page_dimensions(doc, page_id);

            let esc = escape_pdf_string(&bates_str);
            let stream_str = format!(
                "q\nBT\n/{} 10 Tf\n0.2 0.2 0.2 rg\n1 0 0 1 {:.2} 24.0 Tm\n({}) Tj\nET\nQ\n",
                font_alias,
                w - 160.0,
                esc
            );
            let _ = append_page_content(doc, page_id, stream_str.into_bytes());
        }
    }
    Ok(())
}

/// Adds a clickable hyperlink annotation to a page.
pub fn add_link(
    doc: &mut LopdfDoc,
    page_index: usize,
    rect: Rect,
    url: &str,
) -> Result<(), EditError> {
    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let &page_id = page_map
        .get(&page_num)
        .ok_or(EditError::PageOutOfBounds(page_index))?;

    let mut action_dict = Dictionary::new();
    action_dict.set(b"S", Object::Name(b"URI".to_vec()));
    action_dict.set(
        b"URI",
        Object::String(url.as_bytes().to_vec(), lopdf::StringFormat::Literal),
    );

    let mut annot_dict = Dictionary::new();
    annot_dict.set(b"Type", Object::Name(b"Annot".to_vec()));
    annot_dict.set(b"Subtype", Object::Name(b"Link".to_vec()));
    annot_dict.set(
        b"Rect",
        Object::Array(vec![
            Object::Real(rect.min_x),
            Object::Real(rect.min_y),
            Object::Real(rect.max_x),
            Object::Real(rect.max_y),
        ]),
    );
    annot_dict.set(b"A", Object::Dictionary(action_dict));
    annot_dict.set(
        b"Border",
        Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(0),
        ]),
    );

    let annot_id = doc.add_object(annot_dict);

    let page = doc
        .get_object_mut(page_id)
        .map_err(EditError::Lopdf)?
        .as_dict_mut()
        .map_err(EditError::Lopdf)?;
    match page.get_mut(b"Annots") {
        Ok(Object::Array(ref mut arr)) => {
            arr.push(Object::Reference(annot_id));
        }
        _ => {
            page.set(b"Annots", Object::Array(vec![Object::Reference(annot_id)]));
        }
    }

    Ok(())
}

/// Represents an editable text block extracted from a page.
#[derive(Clone, Debug, PartialEq)]
pub struct PageTextBlock {
    pub id: usize,
    pub content_id: Option<ObjectId>,
    pub op_index: usize,
    pub text: String,
    /// Bounding rectangle in page points: [x, y, width, height] where y is from bottom in PDF points
    pub rect: [f32; 4],
    pub font_size: f32,
    pub font_name: String,
}

fn decode_pdf_string_object(obj: &Object) -> String {
    match obj {
        Object::String(bytes, _) => decode_pdf_bytes(bytes),
        _ => String::new(),
    }
}

fn decode_pdf_bytes(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let u16s: Vec<u16> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&[b0, b1]| u16::from_be_bytes([b0, b1]))
            .collect();
        String::from_utf16_lossy(&u16s)
    } else {
        String::from_utf8(bytes.to_vec())
            .unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect())
    }
}

/// Extracts all text blocks with coordinates from a page's content streams.
pub fn extract_page_text_blocks(doc: &LopdfDoc, page_index: usize) -> Vec<PageTextBlock> {
    let mut blocks = Vec::new();
    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let Some(&page_id) = page_map.get(&page_num) else {
        return blocks;
    };

    let content_ids: Vec<ObjectId> = {
        let Ok(page) = doc.get_object(page_id).and_then(Object::as_dict) else {
            return blocks;
        };
        match page.get(b"Contents") {
            Ok(Object::Reference(id)) => vec![*id],
            Ok(Object::Array(arr)) => arr.iter().filter_map(|o| o.as_reference().ok()).collect(),
            _ => Vec::new(),
        }
    };

    let mut block_id = 0;
    for cid in content_ids {
        if let Ok(Object::Stream(stream)) = doc.get_object(cid) {
            let data = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            if let Ok(content) = lopdf::content::Content::decode(&data) {
                let mut current_font_size = 12.0;
                let mut current_font_name = String::new();
                let mut tm_x = 0.0;
                let mut tm_y = 0.0;
                let mut tlm_x = 0.0;
                let mut tlm_y = 0.0;

                for (op_idx, op) in content.operations.iter().enumerate() {
                    match op.operator.as_str() {
                        "BT" => {
                            tm_x = 0.0;
                            tm_y = 0.0;
                            tlm_x = 0.0;
                            tlm_y = 0.0;
                        }
                        "Tf" => {
                            if let Some(name_obj) = op.operands.first() {
                                if let Ok(name) = name_obj.as_name() {
                                    current_font_name = String::from_utf8_lossy(name).to_string();
                                }
                            }
                            if let Some(size_obj) = op.operands.get(1) {
                                if let Some(s) = get_f32_from_obj(size_obj) {
                                    current_font_size = s;
                                }
                            }
                        }
                        "Tm" => {
                            if op.operands.len() >= 6 {
                                let e = get_f32_from_obj(&op.operands[4]).unwrap_or(0.0);
                                let f = get_f32_from_obj(&op.operands[5]).unwrap_or(0.0);
                                tm_x = e;
                                tm_y = f;
                                tlm_x = e;
                                tlm_y = f;
                            }
                        }
                        "Td" | "TD" => {
                            if op.operands.len() >= 2 {
                                let tx = get_f32_from_obj(&op.operands[0]).unwrap_or(0.0);
                                let ty = get_f32_from_obj(&op.operands[1]).unwrap_or(0.0);
                                tlm_x += tx;
                                tlm_y += ty;
                                tm_x = tlm_x;
                                tm_y = tlm_y;
                            }
                        }
                        "T*" => {
                            tlm_y -= current_font_size * 1.2;
                            tm_x = tlm_x;
                            tm_y = tlm_y;
                        }
                        "Tj" | "'" | "\"" => {
                            let text_obj = if op.operator == "\"" {
                                op.operands.get(2)
                            } else {
                                op.operands.first()
                            };
                            if let Some(t_obj) = text_obj {
                                let text_str = decode_pdf_string_object(t_obj);
                                if !text_str.trim().is_empty() {
                                    let width_est = (text_str.chars().count() as f32)
                                        * (current_font_size * 0.55);
                                    let height_est = current_font_size * 1.25;
                                    blocks.push(PageTextBlock {
                                        id: block_id,
                                        content_id: Some(cid),
                                        op_index: op_idx,
                                        text: text_str,
                                        rect: [
                                            tm_x,
                                            tm_y,
                                            width_est.max(20.0),
                                            height_est.max(12.0),
                                        ],
                                        font_size: current_font_size,
                                        font_name: current_font_name.clone(),
                                    });
                                    block_id += 1;
                                }
                            }
                        }
                        "TJ" => {
                            if let Some(arr_obj) = op.operands.first() {
                                if let Ok(arr) = arr_obj.as_array() {
                                    let mut line_text = String::new();
                                    for item in arr {
                                        if let Object::String(ref b, _) = item {
                                            let s = decode_pdf_bytes(b);
                                            line_text.push_str(&s);
                                        } else if let Some(kern) = get_f32_from_obj(item) {
                                            if kern < -150.0 && !line_text.ends_with(' ') {
                                                line_text.push(' ');
                                            }
                                        }
                                    }
                                    if !line_text.trim().is_empty() {
                                        let width_est = (line_text.chars().count() as f32)
                                            * (current_font_size * 0.55);
                                        let height_est = current_font_size * 1.25;
                                        blocks.push(PageTextBlock {
                                            id: block_id,
                                            content_id: Some(cid),
                                            op_index: op_idx,
                                            text: line_text,
                                            rect: [
                                                tm_x,
                                                tm_y,
                                                width_est.max(20.0),
                                                height_est.max(12.0),
                                            ],
                                            font_size: current_font_size,
                                            font_name: current_font_name.clone(),
                                        });
                                        block_id += 1;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    blocks
}

/// Replaces a targeted text block in a page.
/// Attempts exact in-place operator replacement first; falls back safely to overlay replacement if exact stream editing cannot be completed.
pub fn replace_text_block(
    doc: &mut LopdfDoc,
    page_index: usize,
    block: &PageTextBlock,
    new_text: &str,
) -> Result<(), EditError> {
    // 1. Try exact in-place operator replacement if content_id is known
    if let Some(cid) = block.content_id {
        if let Ok(Object::Stream(ref mut stream)) = doc.get_object_mut(cid) {
            let data = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            if let Ok(mut content) = lopdf::content::Content::decode(&data) {
                if block.op_index < content.operations.len() {
                    let op = &mut content.operations[block.op_index];
                    let mut replaced = false;
                    match op.operator.as_str() {
                        "Tj" | "'" => {
                            if let Some(Object::String(bytes, fmt)) = op.operands.first_mut() {
                                *bytes = encode_pdf_string_bytes(new_text);
                                *fmt = lopdf::StringFormat::Literal;
                                replaced = true;
                            }
                        }
                        "\"" => {
                            if let Some(Object::String(bytes, fmt)) = op.operands.get_mut(2) {
                                *bytes = encode_pdf_string_bytes(new_text);
                                *fmt = lopdf::StringFormat::Literal;
                                replaced = true;
                            }
                        }
                        "TJ" => {
                            if let Some(Object::Array(arr)) = op.operands.first_mut() {
                                arr.clear();
                                arr.push(Object::String(
                                    encode_pdf_string_bytes(new_text),
                                    lopdf::StringFormat::Literal,
                                ));
                                replaced = true;
                            }
                        }
                        _ => {}
                    }
                    if replaced {
                        if let Ok(encoded) = content.encode() {
                            stream.set_plain_content(encoded);
                            stream.dict.remove(b"Filter");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    // 2. Fall back to clean overlay text replacement:
    // Mask underlying region with background rectangle and append replacement text
    replace_text_block_overlay(doc, page_index, block, new_text, true)
}

/// Performs overlay text replacement over a text block region.
/// If `mask_bg` is true, paints a background mask over the block rectangle before appending the replacement text.
pub fn replace_text_block_overlay(
    doc: &mut LopdfDoc,
    page_index: usize,
    block: &PageTextBlock,
    new_text: &str,
    mask_bg: bool,
) -> Result<(), EditError> {
    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let &page_id = page_map
        .get(&page_num)
        .ok_or(EditError::PageOutOfBounds(page_index))?;

    let font_alias = ensure_page_font(doc, page_id, StandardFont::Helvetica)?;
    let escaped = escape_pdf_string(new_text);

    let [x, y, w, h] = block.rect;
    let mut stream_str = String::new();

    if mask_bg {
        // Draw white mask rectangle to cover original text cleanly without touching neighboring areas
        stream_str.push_str(&format!(
            "q\n1 1 1 rg\n{:.2} {:.2} {:.2} {:.2} re\nf\nQ\n",
            x - 1.0,
            y - 2.0,
            w + 4.0,
            h + 4.0
        ));
    }

    if !new_text.is_empty() {
        stream_str.push_str(&format!(
            "q\nBT\n/{} {} Tf\n0 0 0 rg\n1 0 0 1 {:.2} {:.2} Tm\n({}) Tj\nET\nQ\n",
            font_alias, block.font_size, x, y, escaped
        ));
    }

    append_page_content(doc, page_id, stream_str.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_page_doc(content_bytes: &[u8]) -> (LopdfDoc, ObjectId) {
        let mut doc = LopdfDoc::with_version("1.7");
        let stream = lopdf::Stream::new(lopdf::Dictionary::new(), content_bytes.to_vec());
        let stream_id = doc.add_object(stream);

        let mut page_dict = lopdf::Dictionary::new();
        page_dict.set(b"Type", Object::Name(b"Page".to_vec()));
        page_dict.set(
            b"MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        );
        page_dict.set(b"Contents", Object::Reference(stream_id));
        let page_id = doc.add_object(page_dict);

        let mut pages_dict = lopdf::Dictionary::new();
        pages_dict.set(b"Type", Object::Name(b"Pages".to_vec()));
        pages_dict.set(b"Kids", Object::Array(vec![Object::Reference(page_id)]));
        pages_dict.set(b"Count", Object::Integer(1));
        let pages_id = doc.add_object(pages_dict);

        if let Ok(p) = doc.get_object_mut(page_id).and_then(Object::as_dict_mut) {
            p.set(b"Parent", Object::Reference(pages_id));
        }

        let mut root_dict = lopdf::Dictionary::new();
        root_dict.set(b"Type", Object::Name(b"Catalog".to_vec()));
        root_dict.set(b"Pages", Object::Reference(pages_id));
        let root_id = doc.add_object(root_dict);

        doc.trailer.set(b"Root", Object::Reference(root_id));
        (doc, page_id)
    }

    #[test]
    fn test_search_and_replace_text_in_page_tj() {
        let raw_stream = b"BT /F1 12 Tf 100 200 Td (Hello World from RustPDF) Tj ET";
        let (mut doc, _) = create_test_page_doc(raw_stream);

        let count = search_and_replace_text_in_page(&mut doc, 0, "World", "Universe").unwrap();
        assert_eq!(count, 1);

        let blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, "Hello Universe from RustPDF");
    }

    #[test]
    fn test_search_and_replace_text_in_page_tj_array() {
        let raw_stream = b"BT /F1 12 Tf 100 200 Td [(Hello ) -200 (World)] TJ ET";
        let (mut doc, _) = create_test_page_doc(raw_stream);

        let count = search_and_replace_text_in_page(&mut doc, 0, "World", "Earth").unwrap();
        assert_eq!(count, 1);

        let blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].text.contains("Earth"));
    }

    #[test]
    fn test_search_and_replace_unicode_roundtrip() {
        let raw_stream = b"BT /F1 12 Tf 100 200 Td (Copyright 2026) Tj ET";
        let (mut doc, _) = create_test_page_doc(raw_stream);

        let count = search_and_replace_text_in_page(&mut doc, 0, "Copyright", "Copyright \u{00A9}")
            .unwrap();
        assert_eq!(count, 1);

        let blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, "Copyright \u{00A9} 2026");
    }

    #[test]
    fn test_replace_specific_text_block_in_place() {
        let raw_stream = b"BT /F1 12 Tf 100 200 Td (Repeat) Tj 0 -20 Td (Repeat) Tj ET";
        let (mut doc, _) = create_test_page_doc(raw_stream);

        let blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].text, "Repeat");
        assert_eq!(blocks[1].text, "Repeat");

        // Target ONLY the second block by its unique op_index
        replace_text_block(&mut doc, 0, &blocks[1], "ModifiedSecond").unwrap();

        let updated_blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(updated_blocks.len(), 2);
        assert_eq!(updated_blocks[0].text, "Repeat");
        assert_eq!(updated_blocks[1].text, "ModifiedSecond");
    }

    #[test]
    fn test_replace_text_block_overlay() {
        let raw_stream = b"BT /F1 12 Tf 100 200 Td (Original Line) Tj ET";
        let (mut doc, _) = create_test_page_doc(raw_stream);

        let blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(blocks.len(), 1);

        replace_text_block_overlay(&mut doc, 0, &blocks[0], "Covered Line", true).unwrap();

        let page_map = doc.get_pages();
        let page_id = *page_map.get(&1).unwrap();
        let page_dict = doc.get_object(page_id).unwrap().as_dict().unwrap();
        let contents = page_dict.get(b"Contents").unwrap();
        match contents {
            Object::Array(arr) => assert!(arr.len() >= 2),
            Object::Reference(_) => {}
            _ => panic!("Expected contents to be reference or array"),
        }
    }
}
