//! rustpdf-forms — Interactive PDF AcroForm field inspection, value editing, and resetting.

use lopdf::{Document as LopdfDoc, Object, StringFormat};
use rustpdf_cos::helpers::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormError {
    #[error("lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("field '{0}' not found")]
    FieldNotFound(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum FieldKind {
    Text,
    Checkbox,
    Radio,
    Choice,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub kind: FieldKind,
    pub value: Option<String>,
    pub default_value: Option<String>,
    pub read_only: bool,
    pub options: Vec<String>,
    pub object_id: (u32, u16),
}

/// Retrieves all interactive form fields defined in the document's /AcroForm.
pub fn get_form_fields(doc: &LopdfDoc) -> Vec<FormField> {
    let catalog = match doc.trailer.get(b"Root").ok().and_then(|r| doc.get_object(r.as_reference().ok()?).ok()) {
        Some(Object::Dictionary(d)) => d,
        _ => return Vec::new(),
    };

    let acro_form = match catalog.get(b"AcroForm").ok().and_then(|r| doc.get_object(r.as_reference().ok()?).ok()) {
        Some(Object::Dictionary(d)) => d,
        _ => return Vec::new(),
    };

    let fields_arr = match acro_form.get(b"Fields").ok() {
        Some(Object::Array(a)) => a,
        Some(Object::Reference(r)) => match doc.get_object(*r).ok() {
            Some(Object::Array(a)) => a,
            _ => return Vec::new(),
        },
        _ => return Vec::new(),
    };

    let mut out = Vec::new();

    for field_ref in fields_arr {
        let obj_id = match field_ref.as_reference() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let field_dict = match doc.get_object(obj_id).and_then(|o| o.as_dict()) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let name = field_dict
            .get(b"T")
            .ok()
            .and_then(to_string_lossy)
            .unwrap_or_else(|| "UnnamedField".to_string());

        let ft = field_dict.get(b"FT").ok().and_then(|o| o.as_name().ok());
        let kind = match ft {
            Some(b"Tx") => FieldKind::Text,
            Some(b"Btn") => {
                // Ff flag bit 16 = radio, otherwise checkbox
                let ff = field_dict.get(b"Ff").ok().and_then(to_i64).unwrap_or(0);
                if (ff & (1 << 15)) != 0 {
                    FieldKind::Radio
                } else {
                    FieldKind::Checkbox
                }
            }
            Some(b"Ch") => FieldKind::Choice,
            _ => FieldKind::Unknown,
        };

        let value = field_dict.get(b"V").ok().and_then(to_string_lossy);
        let default_value = field_dict.get(b"DV").ok().and_then(to_string_lossy);

        let read_only = field_dict
            .get(b"Ff")
            .ok()
            .and_then(to_i64)
            .map(|ff| (ff & 1) != 0)
            .unwrap_or(false);

        let mut options = Vec::new();
        if let Some(Object::Array(opt_arr)) = field_dict.get(b"Opt").ok() {
            for o in opt_arr {
                if let Some(s) = to_string_lossy(o) {
                    options.push(s);
                }
            }
        }

        out.push(FormField {
            name,
            kind,
            value,
            default_value,
            read_only,
            options,
            object_id: obj_id,
        });
    }

    out
}

/// Updates the value of a named form field.
pub fn set_field_value(
    doc: &mut LopdfDoc,
    field_name: &str,
    new_val: &str,
) -> Result<(), FormError> {
    let fields = get_form_fields(doc);
    let target = fields
        .into_iter()
        .find(|f| f.name == field_name)
        .ok_or_else(|| FormError::FieldNotFound(field_name.to_string()))?;

    if let Ok(Object::Dictionary(ref mut dict)) = doc.get_object_mut(target.object_id) {
        dict.set(
            b"V",
            Object::String(new_val.as_bytes().to_vec(), StringFormat::Literal),
        );
    }

    Ok(())
}

/// Resets a named form field to its default value (/DV), or removes /V if no default exists.
pub fn reset_field(
    doc: &mut LopdfDoc,
    field_name: &str,
) -> Result<(), FormError> {
    let fields = get_form_fields(doc);
    let target = fields
        .into_iter()
        .find(|f| f.name == field_name)
        .ok_or_else(|| FormError::FieldNotFound(field_name.to_string()))?;

    if let Ok(Object::Dictionary(ref mut dict)) = doc.get_object_mut(target.object_id) {
        if let Some(dv) = target.default_value {
            dict.set(
                b"V",
                Object::String(dv.as_bytes().to_vec(), StringFormat::Literal),
            );
        } else {
            dict.remove(b"V");
        }
    }

    Ok(())
}

/// Resets all interactive form fields to their defaults. Returns count of reset fields.
pub fn reset_all_fields(doc: &mut LopdfDoc) -> Result<usize, FormError> {
    let fields = get_form_fields(doc);
    let count = fields.len();
    for f in &fields {
        let _ = reset_field(doc, &f.name);
    }
    Ok(count)
}

/// Exports all form field names and their current values as a JSON map.
pub fn export_form_data_json(doc: &LopdfDoc) -> Result<String, FormError> {
    let fields = get_form_fields(doc);
    let mut map = std::collections::BTreeMap::new();
    for f in fields {
        map.insert(f.name, f.value.unwrap_or_default());
    }
    let json = serde_json::to_string_pretty(&map)?;
    Ok(json)
}

/// Imports form field key-value pairs from a JSON map and populates matching fields.
pub fn import_form_data_json(doc: &mut LopdfDoc, json_str: &str) -> Result<usize, FormError> {
    let map: std::collections::BTreeMap<String, String> = serde_json::from_str(json_str)?;
    let mut updated = 0;
    for (k, v) in map {
        if set_field_value(doc, &k, &v).is_ok() {
            updated += 1;
        }
    }
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_form_fields() {
        let doc = LopdfDoc::with_version("1.7");
        let fields = get_form_fields(&doc);
        assert_eq!(fields.len(), 0);
        let json = export_form_data_json(&doc).unwrap();
        assert_eq!(json.trim(), "{}");
    }
}
