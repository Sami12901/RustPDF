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
    #[error("field '{0}' is read-only")]
    ReadOnlyField(String),
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
    let catalog = match doc
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|r| doc.get_object(r.as_reference().ok()?).ok())
    {
        Some(Object::Dictionary(d)) => d,
        _ => return Vec::new(),
    };

    let acro_form = match catalog
        .get(b"AcroForm")
        .ok()
        .and_then(|r| doc.get_object(r.as_reference().ok()?).ok())
    {
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
        if let Ok(Object::Array(opt_arr)) = field_dict.get(b"Opt") {
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

/// Creates the correct PDF object representation for a form field value.
fn field_value_object(field: &FormField, value: &str) -> Object {
    match field.kind {
        FieldKind::Checkbox | FieldKind::Radio => Object::Name(value.as_bytes().to_vec()),
        _ => Object::String(value.as_bytes().to_vec(), StringFormat::Literal),
    }
}

/// Marks AcroForm widget appearances as needing regeneration.
fn mark_need_appearances(doc: &mut LopdfDoc) -> Result<(), FormError> {
    let root_id = doc
        .trailer
        .get(b"Root")?
        .as_reference()?;
    let acro_form_id = {
        let root = doc.get_object(root_id)?.as_dict()?;
        match root.get(b"AcroForm")? {
            Object::Reference(id) => Some(*id),
            Object::Dictionary(_) => None,
            other => {
                other.as_dict()?;
                None
            }
        }
    };

    if let Some(id) = acro_form_id {
        doc.get_object_mut(id)?
            .as_dict_mut()?
            .set(b"NeedAppearances", Object::Boolean(true));
    } else {
        doc.get_object_mut(root_id)?
            .as_dict_mut()?
            .get_mut(b"AcroForm")?
            .as_dict_mut()?
            .set(b"NeedAppearances", Object::Boolean(true));
    }

    Ok(())
}

/// Updates the value of a named form field.
///
/// The update is transactional: the caller's document is only replaced after
/// both the field value and the appearance-refresh flag have been written.
pub fn set_field_value(
    doc: &mut LopdfDoc,
    field_name: &str,
    new_val: &str,
) -> Result<(), FormError> {
    let target = get_form_fields(doc)
        .into_iter()
        .find(|field| field.name == field_name)
        .ok_or_else(|| FormError::FieldNotFound(field_name.to_string()))?;

    if target.read_only {
        return Err(FormError::ReadOnlyField(field_name.to_string()));
    }

    let mut working = doc.clone();
    working
        .get_object_mut(target.object_id)?
        .as_dict_mut()?
        .set(b"V", field_value_object(&target, new_val));
    mark_need_appearances(&mut working)?;
    *doc = working;
    Ok(())
}

/// Resets a named form field to its default value (/DV), or removes /V if no default exists.
pub fn reset_field(doc: &mut LopdfDoc, field_name: &str) -> Result<(), FormError> {
    let target = get_form_fields(doc)
        .into_iter()
        .find(|field| field.name == field_name)
        .ok_or_else(|| FormError::FieldNotFound(field_name.to_string()))?;

    if target.read_only {
        return Err(FormError::ReadOnlyField(field_name.to_string()));
    }

    let mut working = doc.clone();
    {
        let dict = working
            .get_object_mut(target.object_id)?
            .as_dict_mut()?;
        if let Some(value) = target.default_value.as_deref() {
            dict.set(b"V", field_value_object(&target, value));
        } else {
            dict.remove(b"V");
        }
    }
    mark_need_appearances(&mut working)?;
    *doc = working;
    Ok(())
}

/// Resets all writable interactive form fields to their defaults.
/// Returns the number of fields actually reset. The whole operation is atomic.
pub fn reset_all_fields(doc: &mut LopdfDoc) -> Result<usize, FormError> {
    let fields = get_form_fields(doc);
    let mut working = doc.clone();
    let mut count = 0;

    for field in &fields {
        if field.read_only {
            continue;
        }

        let dict = working
            .get_object_mut(field.object_id)?
            .as_dict_mut()?;
        if let Some(value) = field.default_value.as_deref() {
            dict.set(b"V", field_value_object(field, value));
        } else {
            dict.remove(b"V");
        }
        count += 1;
    }

    if count > 0 {
        mark_need_appearances(&mut working)?;
        *doc = working;
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

    fn make_text_form(read_only: bool) -> LopdfDoc {
        let mut doc = LopdfDoc::with_version("1.7");
        let mut field = lopdf::Dictionary::new();
        field.set(b"T", Object::String(b"Name".to_vec(), StringFormat::Literal));
        field.set(b"FT", Object::Name(b"Tx".to_vec()));
        field.set(b"V", Object::String(b"Old".to_vec(), StringFormat::Literal));
        field.set(b"DV", Object::String(b"Default".to_vec(), StringFormat::Literal));
        field.set(b"Ff", Object::Integer(if read_only { 1 } else { 0 }));
        let field_id = doc.add_object(field);

        let mut acro_form = lopdf::Dictionary::new();
        acro_form.set(
            b"Fields",
            Object::Array(vec![Object::Reference(field_id)]),
        );
        let acro_form_id = doc.add_object(acro_form);

        let mut catalog = lopdf::Dictionary::new();
        catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
        catalog.set(b"AcroForm", Object::Reference(acro_form_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set(b"Root", Object::Reference(catalog_id));
        doc
    }

    #[test]
    fn test_set_value_refuses_read_only_fields_without_modifying_document() {
        let mut doc = make_text_form(true);
        assert_eq!(
            set_field_value(&mut doc, "Name", "New"),
            Err(FormError::ReadOnlyField("Name".to_string()))
        );
        let field = get_form_fields(&doc).pop().unwrap();
        assert_eq!(field.value.as_deref(), Some("Old"));
    }

    #[test]
    fn test_set_and_reset_update_appearance_flag_and_value() {
        let mut doc = make_text_form(false);
        set_field_value(&mut doc, "Name", "Updated").unwrap();
        let field = get_form_fields(&doc).pop().unwrap();
        assert_eq!(field.value.as_deref(), Some("Updated"));

        let root_id = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
        let catalog = doc.get_object(root_id).unwrap().as_dict().unwrap();
        let acro_id = catalog.get(b"AcroForm").unwrap().as_reference().unwrap();
        let acro = doc.get_object(acro_id).unwrap().as_dict().unwrap();
        assert_eq!(
            acro.get(b"NeedAppearances").unwrap().as_bool().unwrap(),
            true
        );

        reset_field(&mut doc, "Name").unwrap();
        let field = get_form_fields(&doc).pop().unwrap();
        assert_eq!(field.value.as_deref(), Some("Default"));
    }

    #[test]
    fn test_reset_all_fields_skips_read_only_fields() {
        let mut doc = make_text_form(true);
        assert_eq!(reset_all_fields(&mut doc).unwrap(), 0);
        let field = get_form_fields(&doc).pop().unwrap();
        assert_eq!(field.value.as_deref(), Some("Old"));
    }

}
