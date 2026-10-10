//! rustpdf-cos/security — ISO 32000-1 Standard Security Handler, permissions flags, and encryption headers.

use lopdf::{Document as LopdfDoc, Object};
use serde::{Deserialize, Serialize};

/// Document access permissions conforming to ISO 32000-1 Table 22.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentPermissions {
    pub allow_print: bool,
    pub allow_modify: bool,
    pub allow_copy: bool,
    pub allow_annotate: bool,
    pub allow_fill_forms: bool,
    pub allow_extract: bool,
    pub allow_assemble: bool,
    pub allow_print_high_quality: bool,
}

impl Default for DocumentPermissions {
    fn default() -> Self {
        Self {
            allow_print: true,
            allow_modify: true,
            allow_copy: true,
            allow_annotate: true,
            allow_fill_forms: true,
            allow_extract: true,
            allow_assemble: true,
            allow_print_high_quality: true,
        }
    }
}

impl DocumentPermissions {
    /// Read-only restricted permissions preset.
    pub fn read_only() -> Self {
        Self {
            allow_print: false,
            allow_modify: false,
            allow_copy: false,
            allow_annotate: false,
            allow_fill_forms: false,
            allow_extract: false,
            allow_assemble: false,
            allow_print_high_quality: false,
        }
    }

    /// Converts permission flags to the 32-bit signed integer /P value.
    pub fn to_p_value(&self) -> i32 {
        let mut p: i32 = -4; // High bits 12-32 must be 1, bits 7-8 must be 1
        if !self.allow_print {
            p &= !(1 << 2);
        }
        if !self.allow_modify {
            p &= !(1 << 3);
        }
        if !self.allow_copy {
            p &= !(1 << 4);
        }
        if !self.allow_annotate {
            p &= !(1 << 5);
        }
        if !self.allow_fill_forms {
            p &= !(1 << 8);
        }
        if !self.allow_extract {
            p &= !(1 << 9);
        }
        if !self.allow_assemble {
            p &= !(1 << 10);
        }
        if !self.allow_print_high_quality {
            p &= !(1 << 11);
        }
        p
    }

    /// Parses permission flags from the signed integer /P value.
    pub fn from_p_value(p: i32) -> Self {
        Self {
            allow_print: (p & (1 << 2)) != 0,
            allow_modify: (p & (1 << 3)) != 0,
            allow_copy: (p & (1 << 4)) != 0,
            allow_annotate: (p & (1 << 5)) != 0,
            allow_fill_forms: (p & (1 << 8)) != 0,
            allow_extract: (p & (1 << 9)) != 0,
            allow_assemble: (p & (1 << 10)) != 0,
            allow_print_high_quality: (p & (1 << 11)) != 0,
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SecurityError {
    #[error("Standard PDF encryption (ISO 32000-1 §7.6) is currently unsupported. Fake/unencrypted dictionary injection is disabled to ensure file integrity and avoid false security.")]
    EncryptionUnsupported,
}

/// Checks if full ISO 32000 cryptographic encryption is supported.
pub fn is_encryption_supported() -> bool {
    false
}

/// Applies Standard Security encryption dictionary to protect the document.
///
/// NOTE: Real PDF encryption requires object-level string and stream encryption
/// with cryptographic key derivation (ISO 32000-1 §7.6). Injecting an unencrypted
/// /Encrypt dictionary creates pseudo-protected or corrupted PDFs. This operation
/// is explicitly disabled until a full cryptographic handler is implemented.
pub fn protect_document(
    _doc: &mut LopdfDoc,
    _user_password: &str,
    _owner_password: &str,
    _perms: DocumentPermissions,
) -> Result<(), SecurityError> {
    Err(SecurityError::EncryptionUnsupported)
}

/// Removes encryption protection dictionary from document trailer.
pub fn remove_protection(doc: &mut LopdfDoc) {
    if let Ok(Object::Reference(id)) = doc.trailer.get(b"Encrypt") {
        let enc_id = *id;
        doc.trailer.remove(b"Encrypt");
        doc.objects.remove(&enc_id);
    } else {
        doc.trailer.remove(b"Encrypt");
    }
}

/// Inspects encryption state and permissions from document trailer.
pub fn inspect_protection(doc: &LopdfDoc) -> Option<DocumentPermissions> {
    let enc_dict = match doc.trailer.get(b"Encrypt").ok() {
        Some(Object::Dictionary(d)) => d,
        Some(Object::Reference(id)) => doc.get_object(*id).ok()?.as_dict().ok()?,
        _ => return None,
    };

    let p = match enc_dict.get(b"P").ok() {
        Some(Object::Integer(val)) => *val as i32,
        _ => -4,
    };

    Some(DocumentPermissions::from_p_value(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions_conversion() {
        let perms = DocumentPermissions {
            allow_print: false,
            allow_copy: false,
            ..Default::default()
        };

        let p = perms.to_p_value();
        let decoded = DocumentPermissions::from_p_value(p);
        assert!(!decoded.allow_print);
        assert!(!decoded.allow_copy);
        assert!(decoded.allow_modify);
    }

    #[test]
    fn test_protect_document_rejects_pseudo_encryption() {
        let mut doc = LopdfDoc::with_version("1.7");
        let perms = DocumentPermissions::read_only();
        let res = protect_document(&mut doc, "user123", "ownerSecret", perms);
        assert_eq!(res, Err(SecurityError::EncryptionUnsupported));
        // Ensure no fake /Encrypt dictionary was added to trailer
        assert!(doc.trailer.get(b"Encrypt").is_err());
    }

    #[test]
    fn test_inspect_and_remove_protection() {
        let mut doc = LopdfDoc::with_version("1.7");
        let mut enc = lopdf::Dictionary::new();
        enc.set(b"Filter", Object::Name(b"Standard".to_vec()));
        enc.set(b"P", Object::Integer(DocumentPermissions::read_only().to_p_value() as i64));
        let enc_id = doc.add_object(enc);
        doc.trailer.set(b"Encrypt", Object::Reference(enc_id));

        let detected = inspect_protection(&doc).unwrap();
        assert!(!detected.allow_print);
        assert!(!detected.allow_copy);

        remove_protection(&mut doc);
        assert!(inspect_protection(&doc).is_none());
    }
}
