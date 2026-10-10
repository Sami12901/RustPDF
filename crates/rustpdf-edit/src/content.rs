//! rustpdf-edit/content — PDF content manipulation: text editing/insertion, image embedding, watermarks, headers & footers.

use lopdf::{Dictionary, Document as LopdfDoc, Object, ObjectId, Stream};
use rustpdf_fonts::StandardFont;
use rustpdf_geom::Rect;
use crate::EditError;

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
pub fn ensure_page_font(doc: &mut LopdfDoc, page_id: ObjectId, font: StandardFont) -> Result<String, EditError> {
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

    let page = doc.get_object_mut(page_id).map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?;
    if !page.has(b"Resources") {
        page.set(b"Resources", Object::Dictionary(Dictionary::new()));
    }

    let res_dict = if let Ok(res_ref) = page.get(b"Resources").and_then(Object::as_reference) {
        doc.get_object_mut(res_ref).map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?
    } else {
        page.get_mut(b"Resources").map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?
    };

    if !res_dict.has(b"Font") {
        res_dict.set(b"Font", Object::Dictionary(Dictionary::new()));
    }

    let fonts = res_dict.get_mut(b"Font").map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?;
    fonts.set(alias.as_bytes(), Object::Reference(font_obj_id));

    Ok(alias.to_string())
}

/// Appends a stream of PDF content operators to the page's `/Contents` stream.
pub fn append_page_content(doc: &mut LopdfDoc, page_id: ObjectId, content_bytes: Vec<u8>) -> Result<(), EditError> {
    let stream = Stream::new(Dictionary::new(), content_bytes);
    let stream_id = doc.add_object(stream);

    let page = doc.get_object_mut(page_id).map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?;
    match page.get_mut(b"Contents") {
        Ok(Object::Reference(existing_id)) => {
            let arr = vec![Object::Reference(*existing_id), Object::Reference(stream_id)];
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
pub fn prepend_page_content(doc: &mut LopdfDoc, page_id: ObjectId, content_bytes: Vec<u8>) -> Result<(), EditError> {
    let stream = Stream::new(Dictionary::new(), content_bytes);
    let stream_id = doc.add_object(stream);

    let page = doc.get_object_mut(page_id).map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?;
    match page.get_mut(b"Contents") {
        Ok(Object::Reference(existing_id)) => {
            let arr = vec![Object::Reference(stream_id), Object::Reference(*existing_id)];
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
    let &page_id = page_map.get(&page_num).ok_or(EditError::PageOutOfBounds(page_index))?;

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
    let img = image::load_from_memory(image_bytes).map_err(|e| EditError::ImageError(e.to_string()))?;
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
    let &page_id = page_map.get(&page_num).ok_or(EditError::PageOutOfBounds(page_index))?;

    let img_alias = format!("Im{}", img_id.0);

    let page = doc.get_object_mut(page_id).map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?;
    if !page.has(b"Resources") {
        page.set(b"Resources", Object::Dictionary(Dictionary::new()));
    }
    let res_dict = if let Ok(res_ref) = page.get(b"Resources").and_then(Object::as_reference) {
        doc.get_object_mut(res_ref).map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?
    } else {
        page.get_mut(b"Resources").map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?
    };

    if !res_dict.has(b"XObject") {
        res_dict.set(b"XObject", Object::Dictionary(Dictionary::new()));
    }
    let xobjs = res_dict.get_mut(b"XObject").map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?;
    xobjs.set(img_alias.as_bytes(), Object::Reference(img_id));

    let stream_str = format!(
        "q\n{:.2} 0 0 {:.2} {:.2} {:.2} cm\n/{} Do\nQ\n",
        width_pt, height_pt, x, y, img_alias
    );

    append_page_content(doc, page_id, stream_str.into_bytes())
}

/// Searches and replaces literal string occurrences inside the content streams of a page.
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
    let &page_id = page_map.get(&page_num).ok_or(EditError::PageOutOfBounds(page_index))?;

    let content_ids: Vec<ObjectId> = {
        let page = doc.get_object(page_id).map_err(EditError::Lopdf)?.as_dict().map_err(EditError::Lopdf)?;
        match page.get(b"Contents") {
            Ok(Object::Reference(id)) => vec![*id],
            Ok(Object::Array(arr)) => arr.iter().filter_map(|o| o.as_reference().ok()).collect(),
            _ => Vec::new(),
        }
    };

    let mut replacements_count = 0;
    let target_bytes = target.as_bytes();
    let repl_bytes = replacement.as_bytes();

    for cid in content_ids {
        if let Ok(Object::Stream(ref mut stream)) = doc.get_object_mut(cid) {
            let mut data = if let Ok(decomp) = stream.decompressed_content() {
                decomp
            } else {
                stream.content.clone()
            };

            let mut modified = false;
            let mut i = 0;
            while i + target_bytes.len() <= data.len() {
                if &data[i..i + target_bytes.len()] == target_bytes {
                    data.splice(i..i + target_bytes.len(), repl_bytes.iter().cloned());
                    replacements_count += 1;
                    modified = true;
                    i += repl_bytes.len();
                } else {
                    i += 1;
                }
            }

            if modified {
                stream.set_plain_content(data);
                stream.dict.remove(b"Filter");
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
                let h_rendered = hdr.replace("{n}", &(page_idx + 1).to_string()).replace("{total}", &total_pages.to_string());
                let esc = escape_pdf_string(&h_rendered);
                stream_str.push_str(&format!(
                    "q\nBT\n/{} {} Tf\n{:.3} {:.3} {:.3} rg\n1 0 0 1 36.0 {:.2} Tm\n({}) Tj\nET\nQ\n",
                    font_alias, font_size, color_rgb[0], color_rgb[1], color_rgb[2], h - 28.0, esc
                ));
            }

            if let Some(ftr) = footer_text {
                let f_rendered = ftr.replace("{n}", &(page_idx + 1).to_string()).replace("{total}", &total_pages.to_string());
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
                font_alias, w - 160.0, esc
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
    let &page_id = page_map.get(&page_num).ok_or(EditError::PageOutOfBounds(page_index))?;

    let mut action_dict = Dictionary::new();
    action_dict.set(b"S", Object::Name(b"URI".to_vec()));
    action_dict.set(b"URI", Object::String(url.as_bytes().to_vec(), lopdf::StringFormat::Literal));

    let mut annot_dict = Dictionary::new();
    annot_dict.set(b"Type", Object::Name(b"Annot".to_vec()));
    annot_dict.set(b"Subtype", Object::Name(b"Link".to_vec()));
    annot_dict.set(b"Rect", Object::Array(vec![
        Object::Real(rect.min_x),
        Object::Real(rect.min_y),
        Object::Real(rect.max_x),
        Object::Real(rect.max_y),
    ]));
    annot_dict.set(b"A", Object::Dictionary(action_dict));
    annot_dict.set(b"Border", Object::Array(vec![Object::Integer(0), Object::Integer(0), Object::Integer(0)]));

    let annot_id = doc.add_object(annot_dict);

    let page = doc.get_object_mut(page_id).map_err(EditError::Lopdf)?.as_dict_mut().map_err(EditError::Lopdf)?;
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
