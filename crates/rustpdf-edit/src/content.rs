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

/// Embeds an image (PNG, JPEG, WebP, etc.) onto a page at (x, y) with (width_pt, height_pt) dimensions.
/// Transparent PNGs/WebPs have their alpha channel extracted into a compliant ISO 32000-1 /SMask image dictionary.
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
    let (w, h) = (img.width(), img.height());
    if (w as u64) * (h as u64) > 64_000_000 {
        return Err(EditError::ImageError(
            "image dimensions exceed 64 megapixel security limit".to_string(),
        ));
    }

    let rgba = img.to_rgba8();
    let mut rgb_data = Vec::with_capacity((w * h * 3) as usize);
    let mut alpha_data = Vec::with_capacity((w * h) as usize);
    let mut has_transparency = false;

    for pixel in rgba.pixels() {
        rgb_data.push(pixel[0]);
        rgb_data.push(pixel[1]);
        rgb_data.push(pixel[2]);
        let a = pixel[3];
        alpha_data.push(a);
        if a < 255 {
            has_transparency = true;
        }
    }

    let mut img_dict = Dictionary::new();
    img_dict.set(b"Type", Object::Name(b"XObject".to_vec()));
    img_dict.set(b"Subtype", Object::Name(b"Image".to_vec()));
    img_dict.set(b"Width", Object::Integer(w as i64));
    img_dict.set(b"Height", Object::Integer(h as i64));
    img_dict.set(b"ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    img_dict.set(b"BitsPerComponent", Object::Integer(8));

    // If image has transparent pixels, create a compliant ISO 32000-1 /SMask (Soft Mask) dictionary
    if has_transparency {
        let mut smask_dict = Dictionary::new();
        smask_dict.set(b"Type", Object::Name(b"XObject".to_vec()));
        smask_dict.set(b"Subtype", Object::Name(b"Image".to_vec()));
        smask_dict.set(b"Width", Object::Integer(w as i64));
        smask_dict.set(b"Height", Object::Integer(h as i64));
        smask_dict.set(b"ColorSpace", Object::Name(b"DeviceGray".to_vec()));
        smask_dict.set(b"BitsPerComponent", Object::Integer(8));

        let smask_stream = Stream::new(smask_dict, alpha_data);
        let smask_id = doc.add_object(smask_stream);
        img_dict.set(b"SMask", Object::Reference(smask_id));
    }

    let stream = Stream::new(img_dict, rgb_data);
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

use crate::bijoy::{bijoy_to_unicode, is_likely_bijoy, unicode_to_bijoy};
use crate::cmap::ToUnicodeCMap;
use std::collections::HashMap;

/// Identifies the character encoding scheme of an extracted text block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockEncoding {
    Unicode,
    BijoyAnsi,
    StandardLatin,
}

/// Represents an editable text block extracted from a page.
#[derive(Clone, Debug, PartialEq)]
pub struct PageTextBlock {
    pub id: usize,
    pub content_id: Option<ObjectId>,
    pub op_index: usize,
    pub op_indices: Vec<usize>,
    pub text: String,
    /// Bounding rectangle in page points: [x, y, width, height] where y is from bottom in PDF points
    pub rect: [f32; 4],
    pub font_size: f32,
    pub font_name: String,
    pub encoding: BlockEncoding,
}

fn decode_pdf_string_with_cmap(bytes: &[u8], cmap: Option<&ToUnicodeCMap>) -> String {
    if let Some(cm) = cmap {
        if !cm.char_map.is_empty() {
            return cm.decode_bytes(bytes);
        }
    }
    decode_pdf_bytes(bytes)
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

/// Helper extracting font dictionaries, BaseFont names, and ToUnicode CMaps for a page.
fn get_page_font_cmaps_and_names(
    doc: &LopdfDoc,
    page_id: ObjectId,
) -> (HashMap<String, ToUnicodeCMap>, HashMap<String, String>) {
    let mut cmaps = HashMap::new();
    let mut base_names = HashMap::new();

    let Ok(page_dict) = doc.get_object(page_id).and_then(Object::as_dict) else {
        return (cmaps, base_names);
    };

    let res_dict = if let Ok(res_ref) = page_dict.get(b"Resources").and_then(Object::as_reference) {
        doc.get_object(res_ref).and_then(Object::as_dict).ok()
    } else {
        page_dict.get(b"Resources").and_then(Object::as_dict).ok()
    };

    let Some(res_dict) = res_dict else {
        return (cmaps, base_names);
    };

    let fonts_dict = if let Ok(fonts_ref) = res_dict.get(b"Font").and_then(Object::as_reference) {
        doc.get_object(fonts_ref).and_then(Object::as_dict).ok()
    } else {
        res_dict.get(b"Font").and_then(Object::as_dict).ok()
    };

    let Some(fonts_dict) = fonts_dict else {
        return (cmaps, base_names);
    };

    for (font_key, font_obj) in fonts_dict.iter() {
        let alias = String::from_utf8_lossy(font_key).to_string();
        let font_dict = match font_obj {
            Object::Reference(id) => doc.get_object(*id).and_then(Object::as_dict).ok(),
            Object::Dictionary(d) => Some(d),
            _ => None,
        };

        if let Some(f_dict) = font_dict {
            if let Ok(base_font) = f_dict.get(b"BaseFont").and_then(Object::as_name) {
                base_names.insert(
                    alias.clone(),
                    String::from_utf8_lossy(base_font).to_string(),
                );
            }

            if let Ok(to_uni_ref) = f_dict.get(b"ToUnicode").and_then(Object::as_reference) {
                if let Ok(stream) = doc.get_object(to_uni_ref).and_then(Object::as_stream) {
                    let data = stream
                        .decompressed_content()
                        .unwrap_or_else(|_| stream.content.clone());
                    let cmap = ToUnicodeCMap::parse(&data);
                    cmaps.insert(alias.clone(), cmap);
                }
            } else if let Ok(stream) = f_dict.get(b"ToUnicode").and_then(Object::as_stream) {
                let data = stream
                    .decompressed_content()
                    .unwrap_or_else(|_| stream.content.clone());
                let cmap = ToUnicodeCMap::parse(&data);
                cmaps.insert(alias.clone(), cmap);
            }
        }
    }

    (cmaps, base_names)
}

struct RawTextRun {
    cid: ObjectId,
    op_index: usize,
    text: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font_size: f32,
    font_name: String,
    encoding: BlockEncoding,
}

/// Clusters adjacent raw text runs on the same baseline into cohesive, unified PageTextBlocks.
fn cluster_runs(runs: Vec<RawTextRun>, mut start_id: usize) -> Vec<PageTextBlock> {
    if runs.is_empty() {
        return Vec::new();
    }

    let mut sorted_runs = runs;
    sorted_runs.sort_by(|a, b| {
        let dy = (b.y - a.y).abs();
        if dy < 3.5 {
            a.x.total_cmp(&b.x)
        } else {
            b.y.total_cmp(&a.y)
        }
    });

    let mut clusters: Vec<Vec<RawTextRun>> = Vec::new();

    for run in sorted_runs {
        if let Some(last_cluster) = clusters.last_mut() {
            let last_run = last_cluster.last().unwrap();
            let same_baseline = (run.y - last_run.y).abs() <= 3.5;
            let prev_right = last_run.x + last_run.width;
            let x_gap = run.x - prev_right;
            let close_horizontal = run.x >= prev_right - (run.font_size * 0.3)
                && x_gap <= (run.font_size * 1.5).max(14.0);

            if same_baseline && close_horizontal {
                last_cluster.push(run);
                continue;
            }
        }
        clusters.push(vec![run]);
    }

    let mut blocks = Vec::with_capacity(clusters.len());

    for cluster in clusters {
        if cluster.is_empty() {
            continue;
        }

        let first = &cluster[0];
        let cid = first.cid;
        let mut min_x = first.x;
        let mut max_x = first.x + first.width;
        let mut min_y = first.y;
        let mut max_h = first.height;
        let font_size = first.font_size;
        let font_name = first.font_name.clone();

        let mut combined_text = String::new();
        let mut op_indices = Vec::with_capacity(cluster.len());
        let mut encoding = first.encoding;

        for (i, r) in cluster.iter().enumerate() {
            min_x = min_x.min(r.x);
            max_x = max_x.max(r.x + r.width);
            min_y = min_y.min(r.y);
            max_h = max_h.max(r.height);
            op_indices.push(r.op_index);

            if r.encoding == BlockEncoding::BijoyAnsi {
                encoding = BlockEncoding::BijoyAnsi;
            }

            if i > 0 {
                let prev_r = &cluster[i - 1];
                let gap = r.x - (prev_r.x + prev_r.width);
                if gap > (font_size * 0.18)
                    && !combined_text.ends_with(' ')
                    && !r.text.starts_with(' ')
                {
                    combined_text.push(' ');
                }
            }
            combined_text.push_str(&r.text);
        }

        let primary_op_index = *op_indices.first().unwrap_or(&0);
        let block_w = (max_x - min_x).max(18.0);
        let block_h = max_h.max(12.0);

        blocks.push(PageTextBlock {
            id: start_id,
            content_id: Some(cid),
            op_index: primary_op_index,
            op_indices,
            text: combined_text,
            rect: [min_x, min_y, block_w, block_h],
            font_size,
            font_name,
            encoding,
        });

        start_id += 1;
    }

    blocks
}

/// Extracts all text blocks with coordinates from a page's content streams,
/// decoding /ToUnicode CMaps, resolving Bijoy ANSI to clean Unicode, and clustering lines.
pub fn extract_page_text_blocks(doc: &LopdfDoc, page_index: usize) -> Vec<PageTextBlock> {
    let mut all_blocks = Vec::new();
    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let Some(&page_id) = page_map.get(&page_num) else {
        return all_blocks;
    };

    let (font_cmaps, font_base_names) = get_page_font_cmaps_and_names(doc, page_id);

    let content_ids: Vec<ObjectId> = {
        let Ok(page) = doc.get_object(page_id).and_then(Object::as_dict) else {
            return all_blocks;
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

                let mut stream_runs = Vec::new();

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
                            if let Some(Object::String(bytes, _)) = text_obj {
                                let cmap = font_cmaps.get(&current_font_name);
                                let raw_decoded = decode_pdf_string_with_cmap(bytes, cmap);
                                let base_name = font_base_names
                                    .get(&current_font_name)
                                    .map(|s| s.as_str())
                                    .unwrap_or("");

                                let (final_text, encoding) =
                                    if is_likely_bijoy(&raw_decoded, base_name) {
                                        (bijoy_to_unicode(&raw_decoded), BlockEncoding::BijoyAnsi)
                                    } else {
                                        (raw_decoded, BlockEncoding::Unicode)
                                    };

                                if !final_text.trim().is_empty() {
                                    let char_count = final_text.chars().count() as f32;
                                    let width_est = char_count * (current_font_size * 0.55);
                                    let height_est = current_font_size * 1.25;

                                    stream_runs.push(RawTextRun {
                                        cid,
                                        op_index: op_idx,
                                        text: final_text,
                                        x: tm_x,
                                        y: tm_y,
                                        width: width_est.max(16.0),
                                        height: height_est.max(12.0),
                                        font_size: current_font_size,
                                        font_name: current_font_name.clone(),
                                        encoding,
                                    });

                                    tm_x += width_est;
                                }
                            }
                        }
                        "TJ" => {
                            if let Some(arr_obj) = op.operands.first() {
                                if let Ok(arr) = arr_obj.as_array() {
                                    let mut line_text = String::new();
                                    let cmap = font_cmaps.get(&current_font_name);

                                    for item in arr {
                                        if let Object::String(ref b, _) = item {
                                            let s = decode_pdf_string_with_cmap(b, cmap);
                                            line_text.push_str(&s);
                                        } else if let Some(kern) = get_f32_from_obj(item) {
                                            if kern < -150.0 && !line_text.ends_with(' ') {
                                                line_text.push(' ');
                                            }
                                        }
                                    }

                                    let base_name = font_base_names
                                        .get(&current_font_name)
                                        .map(|s| s.as_str())
                                        .unwrap_or("");

                                    let (final_text, encoding) =
                                        if is_likely_bijoy(&line_text, base_name) {
                                            (bijoy_to_unicode(&line_text), BlockEncoding::BijoyAnsi)
                                        } else {
                                            (line_text, BlockEncoding::Unicode)
                                        };

                                    if !final_text.trim().is_empty() {
                                        let char_count = final_text.chars().count() as f32;
                                        let width_est = char_count * (current_font_size * 0.55);
                                        let height_est = current_font_size * 1.25;

                                        stream_runs.push(RawTextRun {
                                            cid,
                                            op_index: op_idx,
                                            text: final_text,
                                            x: tm_x,
                                            y: tm_y,
                                            width: width_est.max(16.0),
                                            height: height_est.max(12.0),
                                            font_size: current_font_size,
                                            font_name: current_font_name.clone(),
                                            encoding,
                                        });

                                        tm_x += width_est;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Cluster runs into cohesive lines
                let clustered = cluster_runs(stream_runs, block_id);
                block_id += clustered.len();
                all_blocks.extend(clustered);
            }
        }
    }

    all_blocks
}

/// Replaces a targeted text block in a page.
/// Attempts exact in-place operator replacement first; falls back safely to overlay replacement if exact stream editing cannot be completed.
pub fn replace_text_block(
    doc: &mut LopdfDoc,
    page_index: usize,
    block: &PageTextBlock,
    new_text: &str,
) -> Result<(), EditError> {
    // Determine target encoding bytes
    let encoded_bytes = match block.encoding {
        BlockEncoding::BijoyAnsi => unicode_to_bijoy(new_text).into_bytes(),
        _ => encode_pdf_string_bytes(new_text),
    };

    // 1. Try exact in-place operator replacement if content_id is known
    if let Some(cid) = block.content_id {
        if let Ok(Object::Stream(ref mut stream)) = doc.get_object_mut(cid) {
            let data = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            if let Ok(mut content) = lopdf::content::Content::decode(&data) {
                let target_ops = if !block.op_indices.is_empty() {
                    block.op_indices.clone()
                } else {
                    vec![block.op_index]
                };

                let mut replaced_primary = false;

                for (idx_in_cluster, &op_idx) in target_ops.iter().enumerate() {
                    if op_idx < content.operations.len() {
                        let op = &mut content.operations[op_idx];
                        if idx_in_cluster == 0 {
                            // Primary operation receives new text
                            match op.operator.as_str() {
                                "Tj" | "'" => {
                                    if let Some(Object::String(bytes, fmt)) =
                                        op.operands.first_mut()
                                    {
                                        *bytes = encoded_bytes.clone();
                                        *fmt = lopdf::StringFormat::Literal;
                                        replaced_primary = true;
                                    }
                                }
                                "\"" => {
                                    if let Some(Object::String(bytes, fmt)) = op.operands.get_mut(2)
                                    {
                                        *bytes = encoded_bytes.clone();
                                        *fmt = lopdf::StringFormat::Literal;
                                        replaced_primary = true;
                                    }
                                }
                                "TJ" => {
                                    if let Some(Object::Array(arr)) = op.operands.first_mut() {
                                        arr.clear();
                                        arr.push(Object::String(
                                            encoded_bytes.clone(),
                                            lopdf::StringFormat::Literal,
                                        ));
                                        replaced_primary = true;
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            // Secondary clustered operations are blanked out cleanly
                            match op.operator.as_str() {
                                "Tj" | "'" => {
                                    if let Some(Object::String(bytes, _)) = op.operands.first_mut()
                                    {
                                        bytes.clear();
                                    }
                                }
                                "\"" => {
                                    if let Some(Object::String(bytes, _)) = op.operands.get_mut(2) {
                                        bytes.clear();
                                    }
                                }
                                "TJ" => {
                                    if let Some(Object::Array(arr)) = op.operands.first_mut() {
                                        arr.clear();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                if replaced_primary {
                    if let Ok(encoded) = content.encode() {
                        stream.set_plain_content(encoded);
                        stream.dict.remove(b"Filter");
                        return Ok(());
                    }
                }
            }
        }
    }

    // 2. Fall back to clean overlay text replacement:
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

/// Represents an embedded image block on a page.
#[derive(Clone, Debug, PartialEq)]
pub struct PageImageBlock {
    pub id: usize,
    pub content_id: Option<ObjectId>,
    pub op_index: usize,
    pub name: String,
    pub xobject_id: ObjectId,
    /// Bounding rectangle in page points: [x, y, width, height] where y is from bottom in PDF points
    pub rect: [f32; 4],
    pub pixel_width: u32,
    pub pixel_height: u32,
}

#[derive(Clone, Copy, Debug)]
struct Matrix2D {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Matrix2D {
    fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Multiplies m_new * ctm_old according to PDF specification section 8.3.3: CTM' = M * CTM
    fn multiply(&self, ctm: &Matrix2D) -> Matrix2D {
        Matrix2D {
            a: self.a * ctm.a + self.b * ctm.c,
            b: self.a * ctm.b + self.b * ctm.d,
            c: self.c * ctm.a + self.d * ctm.c,
            d: self.c * ctm.b + self.d * ctm.d,
            e: self.e * ctm.a + self.f * ctm.c + ctm.e,
            f: self.e * ctm.b + self.f * ctm.d + ctm.f,
        }
    }

    fn transform(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

/// Helper extracting `/Resources /XObject` dictionaries for a page, walking up `/Parent` if inherited.
fn get_page_image_xobjects(
    doc: &LopdfDoc,
    page_id: ObjectId,
) -> HashMap<Vec<u8>, (ObjectId, u32, u32)> {
    let mut images = HashMap::new();
    let mut current_id = page_id;
    let mut res_dict: Option<&Dictionary> = None;

    for _ in 0..10 {
        if let Ok(dict) = doc.get_object(current_id).and_then(Object::as_dict) {
            if let Ok(res_ref) = dict.get(b"Resources").and_then(Object::as_reference) {
                if let Ok(r) = doc.get_object(res_ref).and_then(Object::as_dict) {
                    res_dict = Some(r);
                    break;
                }
            } else if let Ok(r) = dict.get(b"Resources").and_then(Object::as_dict) {
                res_dict = Some(r);
                break;
            }
            if let Ok(parent_id) = dict.get(b"Parent").and_then(Object::as_reference) {
                current_id = parent_id;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let Some(res_dict) = res_dict else {
        return images;
    };

    let xobjs_dict = if let Ok(xobjs_ref) = res_dict.get(b"XObject").and_then(Object::as_reference)
    {
        doc.get_object(xobjs_ref).and_then(Object::as_dict).ok()
    } else {
        res_dict.get(b"XObject").and_then(Object::as_dict).ok()
    };

    let Some(xobjs_dict) = xobjs_dict else {
        return images;
    };

    for (name_key, val_obj) in xobjs_dict.iter() {
        let xobj_id = match val_obj {
            Object::Reference(id) => *id,
            _ => continue,
        };
        if let Ok(Object::Stream(stream)) = doc.get_object(xobj_id) {
            let subtype = stream
                .dict
                .get(b"Subtype")
                .and_then(Object::as_name)
                .unwrap_or(b"");
            if subtype == b"Image" {
                let w = stream
                    .dict
                    .get(b"Width")
                    .and_then(Object::as_i64)
                    .unwrap_or(0) as u32;
                let h = stream
                    .dict
                    .get(b"Height")
                    .and_then(Object::as_i64)
                    .unwrap_or(0) as u32;
                images.insert(name_key.clone(), (xobj_id, w, h));
            }
        }
    }

    images
}

/// Extracts all image blocks with bounding box coordinates from a page's content streams and resources.
pub fn extract_page_image_blocks(doc: &LopdfDoc, page_index: usize) -> Vec<PageImageBlock> {
    let mut image_blocks = Vec::new();
    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let Some(&page_id) = page_map.get(&page_num) else {
        return image_blocks;
    };

    let image_xobjects = get_page_image_xobjects(doc, page_id);
    if image_xobjects.is_empty() {
        return image_blocks;
    }

    let content_ids: Vec<ObjectId> = {
        let Ok(page) = doc.get_object(page_id).and_then(Object::as_dict) else {
            return image_blocks;
        };
        match page.get(b"Contents") {
            Ok(Object::Reference(id)) => vec![*id],
            Ok(Object::Array(arr)) => arr.iter().filter_map(|o| o.as_reference().ok()).collect(),
            _ => Vec::new(),
        }
    };

    let mut next_id = 0;

    for cid in content_ids {
        if let Ok(Object::Stream(stream)) = doc.get_object(cid) {
            let data = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            if let Ok(content) = lopdf::content::Content::decode(&data) {
                let mut ctm_stack: Vec<Matrix2D> = Vec::new();
                let mut ctm = Matrix2D::identity();

                for (op_idx, op) in content.operations.iter().enumerate() {
                    match op.operator.as_str() {
                        "q" => {
                            ctm_stack.push(ctm);
                        }
                        "Q" => {
                            if let Some(prev) = ctm_stack.pop() {
                                ctm = prev;
                            }
                        }
                        "cm" => {
                            if op.operands.len() >= 6 {
                                let a = get_f32_from_obj(&op.operands[0]).unwrap_or(1.0);
                                let b = get_f32_from_obj(&op.operands[1]).unwrap_or(0.0);
                                let c = get_f32_from_obj(&op.operands[2]).unwrap_or(0.0);
                                let d = get_f32_from_obj(&op.operands[3]).unwrap_or(1.0);
                                let e = get_f32_from_obj(&op.operands[4]).unwrap_or(0.0);
                                let f = get_f32_from_obj(&op.operands[5]).unwrap_or(0.0);
                                let m = Matrix2D { a, b, c, d, e, f };
                                ctm = m.multiply(&ctm);
                            }
                        }
                        "Do" => {
                            if let Some(name_obj) = op.operands.first() {
                                let name_bytes = match name_obj {
                                    Object::Name(bytes) => bytes.as_slice(),
                                    Object::String(bytes, _) => bytes.as_slice(),
                                    _ => &[],
                                };
                                if let Some(&(xobj_id, pixel_w, pixel_h)) =
                                    image_xobjects.get(name_bytes)
                                {
                                    let p0 = ctm.transform(0.0, 0.0);
                                    let p1 = ctm.transform(1.0, 0.0);
                                    let p2 = ctm.transform(1.0, 1.0);
                                    let p3 = ctm.transform(0.0, 1.0);

                                    let min_x = p0.0.min(p1.0).min(p2.0).min(p3.0);
                                    let max_x = p0.0.max(p1.0).max(p2.0).max(p3.0);
                                    let min_y = p0.1.min(p1.1).min(p2.1).min(p3.1);
                                    let max_y = p0.1.max(p1.1).max(p2.1).max(p3.1);
                                    let width = (max_x - min_x).max(1.0);
                                    let height = (max_y - min_y).max(1.0);

                                    image_blocks.push(PageImageBlock {
                                        id: next_id,
                                        content_id: Some(cid),
                                        op_index: op_idx,
                                        name: String::from_utf8_lossy(name_bytes).to_string(),
                                        xobject_id: xobj_id,
                                        rect: [min_x, min_y, width, height],
                                        pixel_width: pixel_w,
                                        pixel_height: pixel_h,
                                    });
                                    next_id += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Fallback for scanned / single-image pages without explicit Do operator in Content stream:
    if image_blocks.is_empty() {
        let (pw, ph) = get_page_dimensions(doc, page_id);
        for (name_bytes, &(xobj_id, pixel_w, pixel_h)) in &image_xobjects {
            image_blocks.push(PageImageBlock {
                id: next_id,
                content_id: None,
                op_index: 0,
                name: String::from_utf8_lossy(name_bytes).to_string(),
                xobject_id: xobj_id,
                rect: [0.0, 0.0, pw, ph],
                pixel_width: pixel_w,
                pixel_height: pixel_h,
            });
            next_id += 1;
        }
    }

    image_blocks
}

/// Replaces an existing image block on a page with new image data in-place, updating XObject stream and SMask.
pub fn replace_page_image(
    doc: &mut LopdfDoc,
    _page_index: usize,
    block: &PageImageBlock,
    new_image_bytes: &[u8],
) -> Result<(), EditError> {
    let img = image::load_from_memory(new_image_bytes)
        .map_err(|e| EditError::ImageError(e.to_string()))?;
    let (w, h) = (img.width(), img.height());
    if (w as u64) * (h as u64) > 64_000_000 {
        return Err(EditError::ImageError(
            "image dimensions exceed 64 megapixel security limit".to_string(),
        ));
    }

    let rgba = img.to_rgba8();
    let mut rgb_data = Vec::with_capacity((w * h * 3) as usize);
    let mut alpha_data = Vec::with_capacity((w * h) as usize);
    let mut has_transparency = false;

    for pixel in rgba.pixels() {
        rgb_data.push(pixel[0]);
        rgb_data.push(pixel[1]);
        rgb_data.push(pixel[2]);
        let a = pixel[3];
        alpha_data.push(a);
        if a < 255 {
            has_transparency = true;
        }
    }

    let existing_smask_ref =
        if let Ok(Object::Stream(existing_stream)) = doc.get_object(block.xobject_id) {
            existing_stream
                .dict
                .get(b"SMask")
                .and_then(Object::as_reference)
                .ok()
        } else {
            None
        };

    let smask_id_opt = if has_transparency {
        let mut smask_dict = Dictionary::new();
        smask_dict.set(b"Type", Object::Name(b"XObject".to_vec()));
        smask_dict.set(b"Subtype", Object::Name(b"Image".to_vec()));
        smask_dict.set(b"Width", Object::Integer(w as i64));
        smask_dict.set(b"Height", Object::Integer(h as i64));
        smask_dict.set(b"ColorSpace", Object::Name(b"DeviceGray".to_vec()));
        smask_dict.set(b"BitsPerComponent", Object::Integer(8));

        let mut smask_stream = Stream::new(smask_dict, alpha_data);
        let _ = smask_stream.compress();

        if let Some(smask_ref) = existing_smask_ref {
            *doc.get_object_mut(smask_ref).map_err(EditError::Lopdf)? =
                Object::Stream(smask_stream);
            Some(smask_ref)
        } else {
            let smask_id = doc.add_object(smask_stream);
            Some(smask_id)
        }
    } else {
        None
    };

    let mut img_dict = Dictionary::new();
    img_dict.set(b"Type", Object::Name(b"XObject".to_vec()));
    img_dict.set(b"Subtype", Object::Name(b"Image".to_vec()));
    img_dict.set(b"Width", Object::Integer(w as i64));
    img_dict.set(b"Height", Object::Integer(h as i64));
    img_dict.set(b"ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    img_dict.set(b"BitsPerComponent", Object::Integer(8));

    if let Some(smask_id) = smask_id_opt {
        img_dict.set(b"SMask", Object::Reference(smask_id));
    }

    let mut new_stream = Stream::new(img_dict, rgb_data);
    let _ = new_stream.compress();

    *doc.get_object_mut(block.xobject_id)
        .map_err(EditError::Lopdf)? = Object::Stream(new_stream);

    Ok(())
}

/// Deletes an image block from a page by removing the `Do` operator from the content stream or masking.
pub fn delete_page_image(
    doc: &mut LopdfDoc,
    page_index: usize,
    block: &PageImageBlock,
) -> Result<(), EditError> {
    let page_map = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let &page_id = page_map
        .get(&page_num)
        .ok_or(EditError::PageOutOfBounds(page_index))?;

    let mut removed_from_stream = false;

    if let Some(cid) = block.content_id {
        if let Ok(Object::Stream(ref mut stream)) = doc.get_object_mut(cid) {
            let data = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            if let Ok(mut content) = lopdf::content::Content::decode(&data) {
                if block.op_index < content.operations.len() {
                    let op = &content.operations[block.op_index];
                    if op.operator == "Do" {
                        content.operations.remove(block.op_index);
                        if let Ok(encoded) = content.encode() {
                            stream.set_plain_content(encoded);
                            stream.dict.remove(b"Filter");
                            removed_from_stream = true;
                        }
                    }
                }
            }
        }
    }

    // Also remove alias from page resources XObject dictionary if present
    if let Ok(page_dict) = doc.get_object_mut(page_id).and_then(Object::as_dict_mut) {
        if let Ok(res_dict) = page_dict
            .get_mut(b"Resources")
            .and_then(Object::as_dict_mut)
        {
            if let Ok(xobjs) = res_dict.get_mut(b"XObject").and_then(Object::as_dict_mut) {
                xobjs.remove(block.name.as_bytes());
            }
        }
    }

    if !removed_from_stream {
        // Fallback: draw white mask rectangle over the image's bounding box
        let [x, y, w, h] = block.rect;
        let stream_str = format!(
            "q\n1 1 1 rg\n{:.2} {:.2} {:.2} {:.2} re\nf\nQ\n",
            x, y, w, h
        );
        append_page_content(doc, page_id, stream_str.into_bytes())?;
    }

    Ok(())
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

    #[test]
    fn test_insert_image_transparent_smask() {
        let (mut doc, _) = create_test_page_doc(b"");

        // Create 2x2 RGBA PNG with alpha
        let mut img_buf = image::RgbaImage::new(2, 2);
        img_buf.put_pixel(0, 0, image::Rgba([255, 0, 0, 128]));
        img_buf.put_pixel(1, 1, image::Rgba([0, 255, 0, 255]));
        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img_buf
            .write_to(&mut cursor, image::ImageFormat::Png)
            .unwrap();

        insert_image_to_page(&mut doc, 0, &png_bytes, 50.0, 50.0, 100.0, 100.0).unwrap();

        // Verify that an image object and an SMask object were added
        let mut found_image_with_smask = false;
        for obj in doc.objects.values() {
            if let Object::Stream(ref stream) = obj {
                if let Ok(subtype) = stream.dict.get(b"Subtype").and_then(Object::as_name) {
                    if subtype == b"Image" && stream.dict.has(b"SMask") {
                        found_image_with_smask = true;
                        break;
                    }
                }
            }
        }
        assert!(
            found_image_with_smask,
            "Expected image to contain an /SMask reference for alpha transparency"
        );
    }

    #[test]
    fn test_extract_and_replace_bijoy_bengali_text() {
        // "KZ cÖKvi ?" in Bijoy encodes "কত প্রকার ?"
        let raw_stream = b"BT /F1 12 Tf 50 400 Td (KZ c\xD6Kvi ?) Tj ET";
        let (mut doc, _) = create_test_page_doc(raw_stream);

        let blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, "কত প্রকার ?");
        assert_eq!(blocks[0].encoding, BlockEncoding::BijoyAnsi);

        // Edit text in Unicode and replace in-place
        replace_text_block(&mut doc, 0, &blocks[0], "কত প্রকার ?").unwrap();

        let updated_blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(updated_blocks.len(), 1);
        assert_eq!(updated_blocks[0].text, "কত প্রকার ?");
    }

    #[test]
    fn test_spatial_clustering_of_fragmented_operators() {
        // Fragmented operators on the same baseline: (Power Suppl) Tj and (y) Tj
        let raw_stream = b"BT /F1 12 Tf 50 400 Td (Power Suppl) Tj (y) Tj ET";
        let (doc, _) = create_test_page_doc(raw_stream);

        let blocks = extract_page_text_blocks(&doc, 0);
        assert_eq!(
            blocks.len(),
            1,
            "Expected fragmented operators to cluster into a single line"
        );
        assert_eq!(blocks[0].text, "Power Supply");
        assert_eq!(blocks[0].op_indices.len(), 2);
    }

    #[test]
    fn test_extract_replace_delete_image_block() {
        let (mut doc, _) = create_test_page_doc(b"");

        // 1. Insert a 2x2 image onto page 0
        let mut img_buf = image::RgbaImage::new(2, 2);
        img_buf.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        img_buf.put_pixel(1, 1, image::Rgba([0, 255, 0, 255]));
        let mut png_bytes = Vec::new();
        img_buf
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .unwrap();

        insert_image_to_page(&mut doc, 0, &png_bytes, 40.0, 60.0, 120.0, 90.0).unwrap();

        // 2. Extract image blocks and verify
        let blocks = extract_page_image_blocks(&doc, 0);
        assert_eq!(blocks.len(), 1, "Expected 1 image block extracted");
        let b = &blocks[0];
        assert_eq!(b.pixel_width, 2);
        assert_eq!(b.pixel_height, 2);
        assert!((b.rect[0] - 40.0).abs() < 1e-2);
        assert!((b.rect[1] - 60.0).abs() < 1e-2);
        assert!((b.rect[2] - 120.0).abs() < 1e-2);
        assert!((b.rect[3] - 90.0).abs() < 1e-2);

        // 3. Replace image block with a 4x4 image
        let mut img_buf2 = image::RgbaImage::new(4, 4);
        img_buf2.put_pixel(0, 0, image::Rgba([0, 0, 255, 255]));
        let mut png_bytes2 = Vec::new();
        img_buf2
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes2),
                image::ImageFormat::Png,
            )
            .unwrap();

        replace_page_image(&mut doc, 0, b, &png_bytes2).unwrap();

        let updated_blocks = extract_page_image_blocks(&doc, 0);
        assert_eq!(updated_blocks.len(), 1);
        assert_eq!(updated_blocks[0].pixel_width, 4);
        assert_eq!(updated_blocks[0].pixel_height, 4);

        // 4. Delete the image block
        delete_page_image(&mut doc, 0, &updated_blocks[0]).unwrap();
        let remaining_blocks = extract_page_image_blocks(&doc, 0);
        assert_eq!(remaining_blocks.len(), 0, "Image block should be removed");
    }
}
