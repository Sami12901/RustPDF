//! rustpdf-model — High-level document model, pages, metadata, outlines, and permissions.

use rustpdf_cos::helpers::*;
use rustpdf_cos::{CosDoc, Dictionary, LopdfDoc, Object, ObjectId, Stream, StringFormat};
use rustpdf_geom::Rect;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("cos error: {0}")]
    Cos(#[from] rustpdf_cos::CosError),
    #[error("missing catalog in PDF")]
    MissingCatalog,
    #[error("page index {0} out of bounds")]
    PageOutOfBounds(usize),
}

/// Metadata attributes associated with the PDF document.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub mod_date: Option<String>,
    pub pdf_version: String,
    pub page_count: usize,
    pub encrypted: bool,
    pub file_size_bytes: usize,
}

impl DocumentMetadata {
    /// Extracts metadata from a CosDoc.
    pub fn from_cos(doc: &CosDoc, file_size: usize) -> Self {
        let mut meta = Self {
            pdf_version: doc.version().to_string(),
            page_count: doc.page_count(),
            file_size_bytes: file_size,
            encrypted: doc.inner().trailer.get(b"Encrypt").is_ok(),
            ..Default::default()
        };

        if let Some(info) = doc.info_dict() {
            meta.title = info.get(b"Title").ok().and_then(to_string_lossy);
            meta.author = info.get(b"Author").ok().and_then(to_string_lossy);
            meta.subject = info.get(b"Subject").ok().and_then(to_string_lossy);
            meta.keywords = info.get(b"Keywords").ok().and_then(to_string_lossy);
            meta.creator = info.get(b"Creator").ok().and_then(to_string_lossy);
            meta.producer = info.get(b"Producer").ok().and_then(to_string_lossy);
            meta.creation_date = info.get(b"CreationDate").ok().and_then(to_string_lossy);
            meta.mod_date = info.get(b"ModDate").ok().and_then(to_string_lossy);
        }

        meta
    }

    /// Writes modified metadata fields back into the CosDoc Info dictionary.
    pub fn write_to_cos(&self, doc: &mut CosDoc) {
        let info_id = match doc.inner().trailer.get(b"Info") {
            Ok(Object::Reference(id)) => *id,
            _ => {
                let new_id = doc.inner_mut().add_object(Dictionary::new());
                doc.inner_mut()
                    .trailer
                    .set(b"Info", Object::Reference(new_id));
                new_id
            }
        };

        let mut dict = if let Some(Object::Dictionary(d)) = doc.inner().objects.get(&info_id) {
            d.clone()
        } else {
            Dictionary::new()
        };

        let set_field = |dict: &mut Dictionary, key: &[u8], val: &Option<String>| {
            if let Some(s) = val {
                dict.set(
                    key,
                    Object::String(s.as_bytes().to_vec(), StringFormat::Literal),
                );
            } else {
                dict.remove(key);
            }
        };

        set_field(&mut dict, b"Title", &self.title);
        set_field(&mut dict, b"Author", &self.author);
        set_field(&mut dict, b"Subject", &self.subject);
        set_field(&mut dict, b"Keywords", &self.keywords);
        set_field(&mut dict, b"Creator", &self.creator);
        set_field(&mut dict, b"Producer", &self.producer);

        doc.inner_mut()
            .objects
            .insert(info_id, Object::Dictionary(dict));
    }
}

/// Information describing a single page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PageModel {
    /// 0-indexed page index
    pub index: usize,
    /// 1-based page number or label
    pub label: String,
    /// Width in points
    pub width: f32,
    /// Height in points
    pub height: f32,
    /// Clockwise rotation in degrees (0, 90, 180, 270)
    pub rotation: i32,
    /// Effective MediaBox in PDF user space
    pub media_box: Rect,
    /// Effective CropBox in PDF user space
    pub crop_box: Rect,
    /// Object ID of this page dictionary
    pub object_id: (u32, u16),
}

impl PageModel {
    pub fn aspect_ratio(&self) -> f32 {
        if self.height > 0.0 {
            self.width / self.height
        } else {
            1.0
        }
    }
}

/// A node in the document bookmark/outline tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutlineNode {
    pub title: String,
    pub page_index: Option<usize>,
    pub children: Vec<OutlineNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PdfLayer {
    pub id: (u32, u16),
    pub name: String,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PdfAttachmentHeader {
    pub filename: String,
    pub size_bytes: usize,
    pub description: Option<String>,
    pub object_id: (u32, u16),
}

/// High-level model of the PDF document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentModel {
    pub metadata: DocumentMetadata,
    pub pages: Vec<PageModel>,
    pub outlines: Vec<OutlineNode>,
    pub layers: Vec<PdfLayer>,
    pub attachments: Vec<PdfAttachmentHeader>,
}

impl DocumentModel {
    /// Builds the document model from a CosDoc.
    pub fn from_cos(doc: &CosDoc, file_size: usize) -> Result<Self, ModelError> {
        let metadata = DocumentMetadata::from_cos(doc, file_size);
        let pages = Self::extract_pages(doc);
        let outlines = Self::extract_outlines(doc, &pages);
        let layers = Self::extract_layers(doc);
        let attachments = Self::extract_attachments(doc);

        Ok(Self {
            metadata,
            pages,
            outlines,
            layers,
            attachments,
        })
    }

    fn extract_pages(doc: &CosDoc) -> Vec<PageModel> {
        let page_map = doc.get_pages();
        let mut pages = Vec::with_capacity(page_map.len());

        for (num, &obj_id) in &page_map {
            let index = (num - 1) as usize;
            let page_dict = match doc.inner().get_object(obj_id).and_then(|o| o.as_dict()) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let media_box = Self::parse_rect(page_dict.get(b"MediaBox").ok())
                .unwrap_or(Rect::new(0.0, 0.0, 612.0, 792.0)); // Default Letter
            let crop_box = Self::parse_rect(page_dict.get(b"CropBox").ok()).unwrap_or(media_box);

            let rotation = page_dict.get(b"Rotate").ok().and_then(to_i64).unwrap_or(0) as i32;

            // Dimensions account for rotation (90 or 270 swaps width and height)
            let (raw_w, raw_h) = (crop_box.width(), crop_box.height());
            let (width, height) = if rotation.abs() % 180 == 90 {
                (raw_h, raw_w)
            } else {
                (raw_w, raw_h)
            };

            pages.push(PageModel {
                index,
                label: num.to_string(),
                width,
                height,
                rotation,
                media_box,
                crop_box,
                object_id: obj_id,
            });
        }

        pages
    }

    fn parse_rect(obj: Option<&Object>) -> Option<Rect> {
        let arr = obj?.as_array().ok()?;
        if arr.len() >= 4 {
            let x0 = to_f32(&arr[0])?;
            let y0 = to_f32(&arr[1])?;
            let x1 = to_f32(&arr[2])?;
            let y1 = to_f32(&arr[3])?;
            Some(Rect::new(x0, y0, x1, y1))
        } else {
            None
        }
    }

    fn extract_outlines(doc: &CosDoc, pages: &[PageModel]) -> Vec<OutlineNode> {
        let catalog = match doc.catalog() {
            Some(c) => c,
            None => return Vec::new(),
        };

        let outlines_ref = match catalog
            .get(b"Outlines")
            .ok()
            .and_then(|o| o.as_reference().ok())
        {
            Some(r) => r,
            None => return Vec::new(),
        };

        let outlines_dict = match doc
            .inner()
            .get_object(outlines_ref)
            .and_then(|o| o.as_dict())
        {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let first_ref = match outlines_dict
            .get(b"First")
            .ok()
            .and_then(|o| o.as_reference().ok())
        {
            Some(r) => r,
            None => return Vec::new(),
        };

        Self::traverse_outline_chain(doc, first_ref, pages, 0)
    }

    fn traverse_outline_chain(
        doc: &CosDoc,
        mut current_ref: ObjectId,
        pages: &[PageModel],
        depth: usize,
    ) -> Vec<OutlineNode> {
        if depth > 32 {
            return Vec::new(); // Prevent infinite loops on cyclic outlines
        }

        let mut items = Vec::new();
        let mut visited = std::collections::HashSet::new();

        while visited.insert(current_ref) {
            let item_dict = match doc
                .inner()
                .get_object(current_ref)
                .and_then(|o| o.as_dict())
            {
                Ok(d) => d,
                Err(_) => break,
            };

            let title = item_dict
                .get(b"Title")
                .ok()
                .and_then(to_string_lossy)
                .unwrap_or_else(|| "Untitled".to_string());

            let page_index = Self::resolve_dest_page(doc, item_dict.get(b"Dest").ok(), pages);

            let children = match item_dict
                .get(b"First")
                .ok()
                .and_then(|o| o.as_reference().ok())
            {
                Some(child_ref) => Self::traverse_outline_chain(doc, child_ref, pages, depth + 1),
                None => Vec::new(),
            };

            items.push(OutlineNode {
                title,
                page_index,
                children,
            });

            match item_dict
                .get(b"Next")
                .ok()
                .and_then(|o| o.as_reference().ok())
            {
                Some(next_ref) => current_ref = next_ref,
                None => break,
            }
        }

        items
    }

    fn resolve_dest_page(
        doc: &CosDoc,
        dest_obj: Option<&Object>,
        pages: &[PageModel],
    ) -> Option<usize> {
        let dest = dest_obj?;
        let target_ref = match dest {
            Object::Array(arr) if !arr.is_empty() => arr[0].as_reference().ok()?,
            Object::Reference(r) => match doc.inner().get_object(*r) {
                Ok(Object::Array(arr)) if !arr.is_empty() => arr[0].as_reference().ok()?,
                _ => return None,
            },
            _ => return None,
        };

        pages.iter().position(|p| p.object_id == target_ref)
    }

    fn extract_layers(doc: &CosDoc) -> Vec<PdfLayer> {
        let catalog = match doc.catalog() {
            Some(c) => c,
            None => return Vec::new(),
        };

        let oc_props = match catalog
            .get(b"OCProperties")
            .ok()
            .and_then(|o| doc.resolve(o))
            .and_then(|o| o.as_dict().ok())
        {
            Some(d) => d,
            None => return Vec::new(),
        };

        let ocgs_arr = match oc_props
            .get(b"OCGs")
            .ok()
            .and_then(|o| doc.resolve(o))
            .and_then(|o| o.as_array().ok())
        {
            Some(a) => a,
            None => return Vec::new(),
        };

        let off_ids: std::collections::HashSet<ObjectId> = oc_props
            .get(b"D")
            .ok()
            .and_then(|o| doc.resolve(o))
            .and_then(|o| o.as_dict().ok())
            .and_then(|d| d.get(b"OFF").ok())
            .and_then(|o| doc.resolve(o))
            .and_then(|o| o.as_array().ok())
            .map(|a| {
                a.iter()
                    .filter_map(|item| item.as_reference().ok())
                    .collect()
            })
            .unwrap_or_default();

        let mut layers = Vec::new();
        for item in ocgs_arr {
            if let Ok(id) = item.as_reference() {
                if let Some(Object::Dictionary(d)) = doc.get_object(id) {
                    let name = d
                        .get(b"Name")
                        .ok()
                        .and_then(to_string_lossy)
                        .unwrap_or_else(|| "Layer".to_string());
                    let visible = !off_ids.contains(&id);
                    layers.push(PdfLayer { id, name, visible });
                }
            }
        }

        layers
    }

    fn extract_attachments(doc: &CosDoc) -> Vec<PdfAttachmentHeader> {
        let catalog = match doc.catalog() {
            Some(c) => c,
            None => return Vec::new(),
        };

        let names_dict = match catalog
            .get(b"Names")
            .ok()
            .and_then(|o| doc.resolve(o))
            .and_then(|o| o.as_dict().ok())
        {
            Some(d) => d,
            None => return Vec::new(),
        };

        let ef_dict = match names_dict
            .get(b"EmbeddedFiles")
            .ok()
            .and_then(|o| doc.resolve(o))
            .and_then(|o| o.as_dict().ok())
        {
            Some(d) => d,
            None => return Vec::new(),
        };

        let names_arr = match ef_dict
            .get(b"Names")
            .ok()
            .and_then(|o| doc.resolve(o))
            .and_then(|o| o.as_array().ok())
        {
            Some(a) => a,
            None => return Vec::new(),
        };

        let mut attachments = Vec::new();
        let mut i = 0;
        while i + 1 < names_arr.len() {
            let label = to_string_lossy(&names_arr[i]).unwrap_or_else(|| "attachment".to_string());
            if let Ok(filespec_id) = names_arr[i + 1].as_reference() {
                if let Some(Object::Dictionary(fs_dict)) = doc.get_object(filespec_id) {
                    let filename = fs_dict
                        .get(b"UF")
                        .or_else(|_| fs_dict.get(b"F"))
                        .ok()
                        .and_then(to_string_lossy)
                        .unwrap_or(label);
                    let desc = fs_dict.get(b"Desc").ok().and_then(to_string_lossy);

                    let size_bytes = fs_dict
                        .get(b"EF")
                        .ok()
                        .and_then(|o| doc.resolve(o))
                        .and_then(|o| o.as_dict().ok())
                        .and_then(|d| d.get(b"F").ok())
                        .and_then(|o| o.as_reference().ok())
                        .and_then(|stream_id| doc.get_object(stream_id))
                        .and_then(|o| o.as_stream().ok())
                        .map(|s| s.content.len())
                        .unwrap_or(0);

                    attachments.push(PdfAttachmentHeader {
                        filename,
                        size_bytes,
                        description: desc,
                        object_id: filespec_id,
                    });
                }
            }
            i += 2;
        }

        attachments
    }
}

/// Attaches an arbitrary file into the document's /EmbeddedFiles name tree.
pub fn add_attachment(
    doc: &mut LopdfDoc,
    filename: &str,
    data: &[u8],
) -> Result<ObjectId, ModelError> {
    let mut stream_dict = Dictionary::new();
    stream_dict.set(b"Type", Object::Name(b"EmbeddedFile".to_vec()));
    let stream = Stream::new(stream_dict, data.to_vec());
    let stream_id = doc.add_object(stream);

    let mut ef_dict = Dictionary::new();
    ef_dict.set(b"F", Object::Reference(stream_id));

    let mut filespec = Dictionary::new();
    filespec.set(b"Type", Object::Name(b"Filespec".to_vec()));
    filespec.set(
        b"F",
        Object::String(filename.as_bytes().to_vec(), StringFormat::Literal),
    );
    filespec.set(
        b"UF",
        Object::String(filename.as_bytes().to_vec(), StringFormat::Literal),
    );
    filespec.set(b"EF", Object::Dictionary(ef_dict));

    let filespec_id = doc.add_object(filespec);

    // Register into Catalog Names -> EmbeddedFiles
    let catalog_id = doc
        .trailer
        .get(b"Root")
        .map_err(|_| ModelError::MissingCatalog)?
        .as_reference()
        .map_err(|_| ModelError::MissingCatalog)?;
    let mut catalog_dict = doc
        .get_object(catalog_id)
        .map_err(|_| ModelError::MissingCatalog)?
        .as_dict()
        .map_err(|_| ModelError::MissingCatalog)?
        .clone();

    let names_id = match catalog_dict.get(b"Names") {
        Ok(Object::Reference(id)) => *id,
        _ => {
            let nid = doc.add_object(Dictionary::new());
            catalog_dict.set(b"Names", Object::Reference(nid));
            doc.objects
                .insert(catalog_id, Object::Dictionary(catalog_dict));
            nid
        }
    };

    let mut names_dict = doc
        .get_object(names_id)
        .and_then(|o| o.as_dict())
        .cloned()
        .unwrap_or_default();
    let ef_id = match names_dict.get(b"EmbeddedFiles") {
        Ok(Object::Reference(id)) => *id,
        _ => {
            let mut ef_new = Dictionary::new();
            ef_new.set(b"Names", Object::Array(Vec::new()));
            let ef_nid = doc.add_object(ef_new);
            names_dict.set(b"EmbeddedFiles", Object::Reference(ef_nid));
            doc.objects.insert(names_id, Object::Dictionary(names_dict));
            ef_nid
        }
    };

    if let Ok(Object::Dictionary(ref mut ef_dict)) = doc.get_object_mut(ef_id) {
        let mut list = match ef_dict.get(b"Names") {
            Ok(Object::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        list.push(Object::String(
            filename.as_bytes().to_vec(),
            StringFormat::Literal,
        ));
        list.push(Object::Reference(filespec_id));
        ef_dict.set(b"Names", Object::Array(list));
    }

    Ok(filespec_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_doc_model() {
        let cos = CosDoc::new();
        let model = DocumentModel::from_cos(&cos, 1024).unwrap();
        assert_eq!(model.metadata.pdf_version, "1.7");
        assert_eq!(model.pages.len(), 0);
        assert_eq!(model.outlines.len(), 0);
        assert_eq!(model.layers.len(), 0);
        assert_eq!(model.attachments.len(), 0);
    }
}
