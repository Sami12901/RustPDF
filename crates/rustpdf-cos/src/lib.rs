//! rustpdf-cos — Carousel Object Structure (COS) low-level document and object tree.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use thiserror::Error;

pub use lopdf::{
    Dictionary, Document as LopdfDoc, Error as LopdfError, Object, ObjectId, Stream,
    StringFormat,
};

pub type Array = Vec<Object>;

pub mod security;

#[derive(Error, Debug)]
pub enum CosError {
    #[error("lopdf error: {0}")]
    Lopdf(#[from] LopdfError),
    #[error("object {0:?} not found")]
    NotFound(ObjectId),
    #[error("expected object to be a {0}")]
    TypeMismatch(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// A wrapper around a low-level PDF document.
#[derive(Clone, Debug)]
pub struct CosDoc {
    inner: LopdfDoc,
}

impl CosDoc {
    /// Creates a blank, empty document.
    pub fn new() -> Self {
        Self {
            inner: LopdfDoc::with_version("1.7"),
        }
    }

    /// Loads a document from raw bytes.
    pub fn load_mem(data: &[u8]) -> Result<Self, CosError> {
        let inner = LopdfDoc::load_mem(data)?;
        Ok(Self { inner })
    }

    /// Loads a document from a reader.
    pub fn load_from<R: Read>(reader: R) -> Result<Self, CosError> {
        let inner = LopdfDoc::load_from(reader)?;
        Ok(Self { inner })
    }

    /// Saves the document into a writer.
    pub fn save_to<W: Write>(&mut self, target: &mut W) -> Result<(), CosError> {
        self.inner.save_to(target)?;
        Ok(())
    }

    /// Returns the underlying lopdf Document reference.
    pub fn inner(&self) -> &LopdfDoc {
        &self.inner
    }

    /// Returns the underlying lopdf Document mutable reference.
    pub fn inner_mut(&mut self) -> &mut LopdfDoc {
        &mut self.inner
    }

    /// Returns the PDF specification version (e.g., "1.7").
    pub fn version(&self) -> &str {
        &self.inner.version
    }

    /// Gets an object by its ID.
    pub fn get_object(&self, id: ObjectId) -> Option<&Object> {
        self.inner.objects.get(&id)
    }

    /// Resolves an object (if it's a reference, follows it to the target).
    pub fn resolve<'a>(&'a self, obj: &'a Object) -> Option<&'a Object> {
        match obj {
            Object::Reference(id) => self.get_object(*id).and_then(|o| self.resolve(o)),
            other => Some(other),
        }
    }

    /// Retrieves all page object IDs in document order: 1-indexed page number -> ObjectId.
    pub fn get_pages(&self) -> BTreeMap<u32, ObjectId> {
        self.inner.get_pages()
    }

    /// Returns the total page count.
    pub fn page_count(&self) -> usize {
        self.inner.get_pages().len()
    }

    /// Returns the catalog (Root) dictionary.
    pub fn catalog(&self) -> Option<&Dictionary> {
        let root_id = self.inner.trailer.get(b"Root").ok()?;
        let resolved = self.resolve(root_id)?;
        resolved.as_dict().ok()
    }

    /// Returns the Info dictionary if present.
    pub fn info_dict(&self) -> Option<&Dictionary> {
        let info_id = self.inner.trailer.get(b"Info").ok()?;
        let resolved = self.resolve(info_id)?;
        resolved.as_dict().ok()
    }
}

impl Default for CosDoc {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for extracting typed data from Objects without panicking.
pub mod helpers {
    use super::*;

    pub fn to_string_lossy(obj: &Object) -> Option<String> {
        match obj {
            Object::String(bytes, _) => {
                // Check if UTF-16BE BOM: \xFE\xFF
                if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
                    let u16_chars: Vec<u16> = bytes[2..]
                        .chunks_exact(2)
                        .map(|chunk| ((chunk[0] as u16) << 8) | (chunk[1] as u16))
                        .collect();
                    String::from_utf16(&u16_chars).ok()
                } else {
                    Some(String::from_utf8_lossy(bytes).to_string())
                }
            }
            Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
            _ => None,
        }
    }

    pub fn to_f32(obj: &Object) -> Option<f32> {
        match obj {
            Object::Real(val) => Some(*val as f32),
            Object::Integer(val) => Some(*val as f32),
            _ => None,
        }
    }

    pub fn to_i64(obj: &Object) -> Option<i64> {
        match obj {
            Object::Integer(val) => Some(*val),
            Object::Real(val) => Some(*val as i64),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_save_empty_doc() {
        let mut doc = CosDoc::new();
        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        assert!(buf.starts_with(b"%PDF-1.7"));
    }

    #[test]
    fn test_helpers() {
        let s = Object::String(b"RustPdf".to_vec(), StringFormat::Literal);
        assert_eq!(helpers::to_string_lossy(&s), Some("RustPdf".to_string()));

        let num = Object::Integer(42);
        assert_eq!(helpers::to_f32(&num), Some(42.0));
        assert_eq!(helpers::to_i64(&num), Some(42));
    }
}
