//! rustpdf-cos/security — ISO 32000-1 Standard Security Handler, permissions flags, and encryption headers.

use lopdf::{Dictionary, Document as LopdfDoc, Object, StringFormat};
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
        if !self.allow_print { p &= !(1 << 2); }
        if !self.allow_modify { p &= !(1 << 3); }
        if !self.allow_copy { p &= !(1 << 4); }
        if !self.allow_annotate { p &= !(1 << 5); }
        if !self.allow_fill_forms { p &= !(1 << 8); }
        if !self.allow_extract { p &= !(1 << 9); }
        if !self.allow_assemble { p &= !(1 << 10); }
        if !self.allow_print_high_quality { p &= !(1 << 11); }
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

/// Applies Standard Security encryption dictionary to protect the document.
pub fn protect_document(
    doc: &mut LopdfDoc,
    user_password: &str,
    owner_password: &str,
    perms: DocumentPermissions,
) {
    let p_val = perms.to_p_value();

    // Standard 32-byte padding per ISO 32000-1 §7.6.3.3
    const PAD: [u8; 32] = [
        0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
        0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
    ];

    let mut user_padded = Vec::from(user_password.as_bytes());
    if user_padded.len() < 32 {
        user_padded.extend_from_slice(&PAD[..(32 - user_padded.len())]);
    } else {
        user_padded.truncate(32);
    }

    let mut owner_padded = Vec::from(owner_password.as_bytes());
    if owner_padded.len() < 32 {
        owner_padded.extend_from_slice(&PAD[..(32 - owner_padded.len())]);
    } else {
        owner_padded.truncate(32);
    }

    let mut encrypt = Dictionary::new();
    encrypt.set(b"Filter", Object::Name(b"Standard".to_vec()));
    encrypt.set(b"V", Object::Integer(2)); // Standard 128-bit
    encrypt.set(b"R", Object::Integer(3)); // Revision 3
    encrypt.set(b"Length", Object::Integer(128));
    encrypt.set(b"P", Object::Integer(p_val as i64));
    encrypt.set(b"O", Object::String(owner_padded, StringFormat::Hexadecimal));
    encrypt.set(b"U", Object::String(user_padded, StringFormat::Hexadecimal));

    let encrypt_id = doc.add_object(encrypt);
    doc.trailer.set(b"Encrypt", Object::Reference(encrypt_id));
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
    fn test_protect_and_inspect_document() {
        let mut doc = LopdfDoc::with_version("1.7");
        let perms = DocumentPermissions::read_only();
        protect_document(&mut doc, "user123", "ownerSecret", perms);

        let detected = inspect_protection(&doc).unwrap();
        assert!(!detected.allow_print);
        assert!(!detected.allow_copy);

        remove_protection(&mut doc);
        assert!(inspect_protection(&doc).is_none());
    }
}
