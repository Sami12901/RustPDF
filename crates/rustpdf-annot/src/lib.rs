//! rustpdf-annot — PDF annotations and markup objects conforming to ISO 32000-1 §12.5.

use lopdf::{Dictionary, Document as LopdfDoc, Object, ObjectId, StringFormat};
use rustpdf_cos::helpers::to_f32;
use rustpdf_geom::Rect;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnnotError {
    #[error("lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
    #[error("page index {0} out of bounds")]
    PageOutOfBounds(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnnotType {
    Highlight,
    Underline,
    StrikeOut,
    Square,
    Circle,
    Line,
    Text, // Sticky note
    Ink,  // Freehand drawing
}

impl AnnotType {
    pub fn pdf_subtype(self) -> &'static [u8] {
        match self {
            Self::Highlight => b"Highlight",
            Self::Underline => b"Underline",
            Self::StrikeOut => b"StrikeOut",
            Self::Square => b"Square",
            Self::Circle => b"Circle",
            Self::Line => b"Line",
            Self::Text => b"Text",
            Self::Ink => b"Ink",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Option<(u32, u16)>,
    pub page_index: usize,
    pub annot_type: AnnotType,
    pub rect: Rect,
    pub color: [f32; 3], // RGB (0.0 to 1.0)
    pub contents: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub ink_paths: Option<Vec<Vec<(f32, f32)>>>,
}

/// Adds a real PDF annotation object to the specified page in the document.
pub fn add_annotation(doc: &mut LopdfDoc, annot: &Annotation) -> Result<ObjectId, AnnotError> {
    let pages = doc.get_pages();
    let page_num = (annot.page_index + 1) as u32;
    let &page_id = pages
        .get(&page_num)
        .ok_or(AnnotError::PageOutOfBounds(annot.page_index))?;

    let mut dict = Dictionary::new();
    dict.set(b"Type", Object::Name(b"Annot".to_vec()));
    dict.set(
        b"Subtype",
        Object::Name(annot.annot_type.pdf_subtype().to_vec()),
    );
    dict.set(
        b"Rect",
        Object::Array(vec![
            Object::Real(annot.rect.min_x),
            Object::Real(annot.rect.min_y),
            Object::Real(annot.rect.max_x),
            Object::Real(annot.rect.max_y),
        ]),
    );

    // RGB color array
    dict.set(
        b"C",
        Object::Array(vec![
            Object::Real(annot.color[0]),
            Object::Real(annot.color[1]),
            Object::Real(annot.color[2]),
        ]),
    );

    if let Some(ref text) = annot.contents {
        dict.set(
            b"Contents",
            Object::String(text.as_bytes().to_vec(), StringFormat::Literal),
        );
    }

    if let Some(ref author) = annot.author {
        dict.set(
            b"T",
            Object::String(author.as_bytes().to_vec(), StringFormat::Literal),
        );
    }

    if let Some(ref date) = annot.date {
        dict.set(
            b"M",
            Object::String(date.as_bytes().to_vec(), StringFormat::Literal),
        );
    }

    if let Some(ref paths) = annot.ink_paths {
        let mut ink_list = Vec::new();
        for path in paths {
            let mut coords = Vec::new();
            for &(x, y) in path {
                coords.push(Object::Real(x));
                coords.push(Object::Real(y));
            }
            ink_list.push(Object::Array(coords));
        }
        dict.set(b"InkList", Object::Array(ink_list));
    }

    let annot_id = doc.add_object(dict);

    // Attach to page's /Annots array
    if let Ok(Object::Dictionary(ref mut page_dict)) = doc.get_object_mut(page_id) {
        let annots_arr = match page_dict.get_mut(b"Annots") {
            Ok(Object::Array(ref mut a)) => a,
            _ => {
                page_dict.set(b"Annots", Object::Array(Vec::new()));
                match page_dict.get_mut(b"Annots") {
                    Ok(Object::Array(ref mut a)) => a,
                    _ => unreachable!(),
                }
            }
        };
        annots_arr.push(Object::Reference(annot_id));
    }

    Ok(annot_id)
}

/// Deletes an annotation by its ObjectId from the document.
pub fn delete_annotation(doc: &mut LopdfDoc, annot_id: (u32, u16)) -> Result<bool, AnnotError> {
    let mut removed = false;
    let pages = doc.get_pages();
    for &page_id in pages.values() {
        if let Ok(Object::Dictionary(ref mut page_dict)) = doc.get_object_mut(page_id) {
            if let Ok(Object::Array(ref mut annots)) = page_dict.get_mut(b"Annots") {
                let initial_len = annots.len();
                annots.retain(|item| match item {
                    Object::Reference(id) => *id != annot_id,
                    _ => true,
                });
                if annots.len() < initial_len {
                    removed = true;
                }
            }
        }
    }
    if removed {
        doc.objects.remove(&annot_id);
    }
    Ok(removed)
}

/// Reads all annotations from the specified page.
pub fn get_annotations(doc: &LopdfDoc, page_index: usize) -> Vec<Annotation> {
    let pages = doc.get_pages();
    let page_num = (page_index + 1) as u32;
    let Some(&page_id) = pages.get(&page_num) else {
        return Vec::new();
    };

    let Ok(page_dict) = doc.get_object(page_id).and_then(|o| o.as_dict()) else {
        return Vec::new();
    };

    let Some(annots_obj) = page_dict.get(b"Annots").ok() else {
        return Vec::new();
    };

    let annots_arr = match annots_obj {
        Object::Array(a) => a,
        Object::Reference(id) => match doc.get_object(*id).and_then(|o| o.as_array()) {
            Ok(a) => a,
            Err(_) => return Vec::new(),
        },
        _ => return Vec::new(),
    };

    let mut out = Vec::new();

    for item in annots_arr {
        let annot_dict = match item {
            Object::Dictionary(d) => d,
            Object::Reference(id) => match doc.get_object(*id).and_then(|o| o.as_dict()) {
                Ok(d) => d,
                Err(_) => continue,
            },
            _ => continue,
        };

        let subtype = match annot_dict
            .get(b"Subtype")
            .ok()
            .and_then(|o| o.as_name().ok())
        {
            Some(b"Highlight") => AnnotType::Highlight,
            Some(b"Underline") => AnnotType::Underline,
            Some(b"StrikeOut") => AnnotType::StrikeOut,
            Some(b"Square") => AnnotType::Square,
            Some(b"Circle") => AnnotType::Circle,
            Some(b"Line") => AnnotType::Line,
            Some(b"Text") => AnnotType::Text,
            Some(b"Ink") => AnnotType::Ink,
            _ => continue,
        };

        let rect = match annot_dict.get(b"Rect").ok().and_then(|o| o.as_array().ok()) {
            Some(a) if a.len() >= 4 => {
                let x0 = to_f32(&a[0]).unwrap_or(0.0);
                let y0 = to_f32(&a[1]).unwrap_or(0.0);
                let x1 = to_f32(&a[2]).unwrap_or(0.0);
                let y1 = to_f32(&a[3]).unwrap_or(0.0);
                Rect::new(x0, y0, x1, y1)
            }
            _ => Rect::ZERO,
        };

        let color = match annot_dict.get(b"C").ok().and_then(|o| o.as_array().ok()) {
            Some(a) if a.len() >= 3 => [
                to_f32(&a[0]).unwrap_or(1.0),
                to_f32(&a[1]).unwrap_or(1.0),
                to_f32(&a[2]).unwrap_or(0.0),
            ],
            _ => [1.0, 0.8, 0.0], // Default amber
        };

        let contents = annot_dict.get(b"Contents").ok().and_then(|o| {
            o.as_str()
                .ok()
                .map(|s| String::from_utf8_lossy(s).to_string())
        });

        let author = annot_dict.get(b"T").ok().and_then(|o| {
            o.as_str()
                .ok()
                .map(|s| String::from_utf8_lossy(s).to_string())
        });

        let date = annot_dict.get(b"M").ok().and_then(|o| {
            o.as_str()
                .ok()
                .map(|s| String::from_utf8_lossy(s).to_string())
        });

        let mut ink_paths = None;
        if let Ok(Object::Array(ref lists)) = annot_dict.get(b"InkList") {
            let mut paths = Vec::new();
            for sub_list in lists {
                if let Ok(coords) = sub_list.as_array() {
                    let mut pts = Vec::new();
                    let mut i = 0;
                    while i + 1 < coords.len() {
                        if let (Some(x), Some(y)) = (to_f32(&coords[i]), to_f32(&coords[i + 1])) {
                            pts.push((x, y));
                        }
                        i += 2;
                    }
                    if !pts.is_empty() {
                        paths.push(pts);
                    }
                }
            }
            if !paths.is_empty() {
                ink_paths = Some(paths);
            }
        }

        let id = match item {
            Object::Reference(ref_id) => Some(*ref_id),
            _ => None,
        };

        out.push(Annotation {
            id,
            page_index,
            annot_type: subtype,
            rect,
            color,
            contents,
            author,
            date,
            ink_paths,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_annotation() {
        let mut doc = LopdfDoc::with_version("1.7");
        rustpdf_organize::insert_blank_page(&mut doc, 0, 612.0, 792.0).unwrap();

        let annot = Annotation {
            id: None,
            page_index: 0,
            annot_type: AnnotType::Highlight,
            rect: Rect::new(100.0, 200.0, 300.0, 220.0),
            color: [1.0, 0.9, 0.0],
            contents: Some("Note here".to_string()),
            author: Some("Saif".to_string()),
            date: Some("2026-10-09".to_string()),
            ink_paths: None,
        };

        let annot_id = add_annotation(&mut doc, &annot).unwrap();

        let loaded = get_annotations(&doc, 0);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].annot_type, AnnotType::Highlight);
        assert_eq!(loaded[0].contents, Some("Note here".to_string()));

        // Delete test
        delete_annotation(&mut doc, annot_id).unwrap();
        assert_eq!(get_annotations(&doc, 0).len(), 0);
    }
}
