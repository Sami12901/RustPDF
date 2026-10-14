//! rustpdf-forms — Interactive PDF AcroForm field inspection, value editing, and resetting.

use lopdf::{Document as LopdfDoc, Object, ObjectId, StringFormat};
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
    #[error("'{value}' is not a valid value for button field '{field}'")]
    InvalidButtonValue { field: String, value: String },
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

#[derive(Clone, Default)]
struct InheritedFieldAttributes {
    field_type: Option<Vec<u8>>,
    flags: i64,
    value: Option<Object>,
    default_value: Option<Object>,
    options: Option<Object>,
}

fn all_child_ids(doc: &LopdfDoc, field_dict: &lopdf::Dictionary) -> Vec<ObjectId> {
    let kids = match field_dict.get(b"Kids").ok() {
        Some(Object::Array(kids)) => kids.clone(),
        Some(Object::Reference(id)) => doc
            .get_object(*id)
            .and_then(|object| object.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => return Vec::new(),
    };
    kids.into_iter()
        .filter_map(|child| child.as_reference().ok().copied())
        .collect()
}

fn field_children(doc: &LopdfDoc, field_dict: &lopdf::Dictionary) -> Vec<ObjectId> {
    all_child_ids(doc, field_dict)
        .into_iter()
        .filter(|child_id| {
            let Ok(child) = doc
                .get_object(*child_id)
                .and_then(|object| object.as_dict())
            else {
                return false;
            };

            let is_widget = child
                .get(b"Subtype")
                .ok()
                .and_then(|value| value.as_name().ok())
                .is_some_and(|name| name == b"Widget");
            let is_named = child.get(b"T").is_ok();
            let has_own_type = child.get(b"FT").is_ok();
            let has_children = child.get(b"Kids").is_ok();

            // Widget annotations are children of a terminal field but are not
            // themselves logical fields unless they also carry field attributes.
            !(is_widget && !is_named && !has_own_type && !has_children)
        })
        .collect()
}

fn resolved_dict<'a>(
    doc: &'a LopdfDoc,
    object: &'a Object,
) -> Option<&'a lopdf::Dictionary> {
    match object {
        Object::Dictionary(dict) => Some(dict),
        Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok(),
        _ => None,
    }
}

fn collect_button_states(
    doc: &LopdfDoc,
    field_id: ObjectId,
    visited: &mut std::collections::HashSet<ObjectId>,
    states: &mut Vec<Vec<u8>>,
) {
    if !visited.insert(field_id) {
        return;
    }
    let Ok(field) = doc.get_object(field_id).and_then(|object| object.as_dict()) else {
        return;
    };
    if let Some(ap) = field.get(b"AP").ok().and_then(|object| resolved_dict(doc, object)) {
        if let Some(normal) = ap.get(b"N").ok().and_then(|object| resolved_dict(doc, object)) {
            for (name, _) in normal.iter() {
                if name.as_slice() != b"Off" && !states.iter().any(|state| state == name) {
                    states.push(name.clone());
                }
            }
        }
    }

    for child_id in all_child_ids(doc, field) {
        collect_button_states(doc, child_id, visited, states);
    }
}

fn button_states_for_field(doc: &LopdfDoc, field_id: ObjectId) -> Vec<Vec<u8>> {
    let mut states = Vec::new();
    collect_button_states(
        doc,
        field_id,
        &mut std::collections::HashSet::new(),
        &mut states,
    );
    states
}

fn resolve_button_value(
    doc: &LopdfDoc,
    field: &FormField,
    requested: &str,
) -> Result<Vec<u8>, FormError> {
    let trimmed = requested.trim();
    let normalized = trimmed.to_ascii_lowercase();

    if matches!(normalized.as_str(), "" | "off" | "false" | "no" | "0") {
        return Ok(b"Off".to_vec());
    }

    let states = button_states_for_field(doc, field.object_id);
    if let Some(state) = states.iter().find(|state| state.as_slice() == trimmed.as_bytes()) {
        return Ok(state.clone());
    }

    if matches!(normalized.as_str(), "on" | "true" | "yes" | "1") {
        return Ok(states
            .first()
            .cloned()
            .unwrap_or_else(|| b"Yes".to_vec()));
    }

    Err(FormError::InvalidButtonValue {
        field: field.name.clone(),
        value: requested.to_string(),
    })
}

fn set_widget_appearance_state(
    doc: &mut LopdfDoc,
    field_id: ObjectId,
    selected_state: &[u8],
    visited: &mut std::collections::HashSet<ObjectId>,
) -> Result<(), FormError> {
    if !visited.insert(field_id) {
        return Ok(());
    }
    let (has_normal_appearance, children, states) = {
        let field = doc.get_object(field_id)?.as_dict()?;
        let has_appearance = field
            .get(b"AP")
            .ok()
            .and_then(|object| resolved_dict(doc, object))
            .and_then(|ap| ap.get(b"N").ok().and_then(|object| resolved_dict(doc, object)))
            .is_some();
        (has_appearance, all_child_ids(doc, field), button_states_for_field(doc, field_id))
    };

    if has_normal_appearance {
        let state = if selected_state == b"Off" || states.iter().any(|value| value == selected_state) {
            selected_state
        } else {
            b"Off"
        };
        doc.get_object_mut(field_id)?
            .as_dict_mut()?
            .set(b"AS", Object::Name(state.to_vec()));
    }

    for child_id in children {
        set_widget_appearance_state(doc, child_id, selected_state, visited)?;
    }
    Ok(())
}

fn collect_fields_recursive(
    doc: &LopdfDoc,
    field_id: ObjectId,
    parent_name: &str,
    inherited: &InheritedFieldAttributes,
    visited: &mut std::collections::HashSet<ObjectId>,
    output: &mut Vec<FormField>,
) {
    if !visited.insert(field_id) {
        return;
    }

    let Ok(field_dict) = doc.get_object(field_id).and_then(|object| object.as_dict()) else {
        return;
    };

    let partial_name = field_dict
        .get(b"T")
        .ok()
        .and_then(to_string_lossy)
        .filter(|name| !name.is_empty());
    let full_name = match partial_name {
        Some(partial) if parent_name.is_empty() => partial,
        Some(partial) => format!("{parent_name}.{partial}"),
        None => parent_name.to_string(),
    };

    let field_type = field_dict
        .get(b"FT")
        .ok()
        .and_then(|value| value.as_name().ok())
        .map(|name| name.to_vec())
        .or_else(|| inherited.field_type.clone());
    let flags = field_dict
        .get(b"Ff")
        .ok()
        .and_then(to_i64)
        .unwrap_or(inherited.flags);
    let value = field_dict
        .get(b"V")
        .ok()
        .cloned()
        .or_else(|| inherited.value.clone());
    let default_value = field_dict
        .get(b"DV")
        .ok()
        .cloned()
        .or_else(|| inherited.default_value.clone());
    let options = field_dict
        .get(b"Opt")
        .ok()
        .cloned()
        .or_else(|| inherited.options.clone());

    let children = field_children(doc, field_dict);
    if !children.is_empty() {
        let next_inherited = InheritedFieldAttributes {
            field_type,
            flags,
            value,
            default_value,
            options,
        };
        for child_id in children {
            collect_fields_recursive(doc, child_id, &full_name, &next_inherited, visited, output);
        }
        return;
    }

    let Some(field_type) = field_type else {
        return;
    };
    let kind = match field_type.as_slice() {
        b"Tx" => FieldKind::Text,
        b"Btn" => {
            // Bit 16 marks radio buttons; otherwise this is a checkbox field.
            if (flags & (1 << 15)) != 0 {
                FieldKind::Radio
            } else {
                FieldKind::Checkbox
            }
        }
        b"Ch" => FieldKind::Choice,
        _ => FieldKind::Unknown,
    };

    let mut parsed_options = Vec::new();
    if let Some(options) = options.as_ref() {
        let option_array = match options {
            Object::Array(values) => Some(values),
            Object::Reference(id) => doc
                .get_object(*id)
                .ok()
                .and_then(|object| object.as_array().ok()),
            _ => None,
        };
        if let Some(option_array) = option_array {
            for option in option_array {
                if let Some(option) = to_string_lossy(option) {
                    parsed_options.push(option);
                } else if let Ok(pair) = option.as_array() {
                    if let Some(export_value) = pair.first().and_then(to_string_lossy) {
                        parsed_options.push(export_value);
                    }
                }
            }
        }
    }

    output.push(FormField {
        name: if full_name.is_empty() {
            format!("UnnamedField-{}-{}", field_id.0, field_id.1)
        } else {
            full_name
        },
        kind,
        value: value.as_ref().and_then(to_string_lossy),
        default_value: default_value.as_ref().and_then(to_string_lossy),
        read_only: (flags & 1) != 0,
        options: parsed_options,
        object_id: field_id,
    });
}

/// Retrieves terminal AcroForm fields, including nested names and inherited field attributes.
/// Widget annotations are excluded from the logical field list, and parent references are
/// cycle-checked to avoid hanging on malformed documents.
pub fn get_form_fields(doc: &LopdfDoc) -> Vec<FormField> {
    let catalog = match doc
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|root| root.as_reference().ok())
        .and_then(|root_id| doc.get_object(root_id).ok())
        .and_then(|root| root.as_dict().ok())
    {
        Some(catalog) => catalog,
        None => return Vec::new(),
    };

    let acro_form = match catalog.get(b"AcroForm").ok() {
        Some(Object::Reference(id)) => {
            match doc.get_object(*id).and_then(|object| object.as_dict()) {
                Ok(dict) => dict,
                Err(_) => return Vec::new(),
            }
        }
        Some(Object::Dictionary(dict)) => dict,
        _ => return Vec::new(),
    };

    let fields = match acro_form.get(b"Fields").ok() {
        Some(Object::Array(fields)) => fields.clone(),
        Some(Object::Reference(id)) => {
            match doc.get_object(*id).and_then(|object| object.as_array()) {
                Ok(fields) => fields.clone(),
                Err(_) => return Vec::new(),
            }
        }
        _ => return Vec::new(),
    };

    let inherited = InheritedFieldAttributes::default();
    let mut visited = std::collections::HashSet::new();
    let mut output = Vec::new();

    for field in fields {
        if let Ok(field_id) = field.as_reference() {
            collect_fields_recursive(doc, *field_id, "", &inherited, &mut visited, &mut output);
        }
    }

    output
}

/// Updates a named AcroForm field transactionally.
pub fn set_field_value(
    doc: &mut LopdfDoc,
    field_name: &str,
    new_val: &str,
) -> Result<(), FormError> {
    let mut candidate = doc.clone();
    set_field_value_in_place(&mut candidate, field_name, new_val)?;
    *doc = candidate;
    Ok(())
}

fn set_field_value_in_place(
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

    match target.kind {
        FieldKind::Checkbox | FieldKind::Radio => {
            let selected_state = resolve_button_value(doc, &target, new_val)?;
            doc.get_object_mut(target.object_id)?
                .as_dict_mut()?
                .set(b"V", Object::Name(selected_state.clone()));
            set_widget_appearance_state(
                doc,
                target.object_id,
                &selected_state,
                &mut std::collections::HashSet::new(),
            )?;
        }
        _ => {
            doc.get_object_mut(target.object_id)?
                .as_dict_mut()?
                .set(
                    b"V",
                    Object::String(new_val.as_bytes().to_vec(), StringFormat::Literal),
                );
        }
    }

    set_need_appearances(doc);
    Ok(())
}

/// Resets a named form field to its default value (/DV), or removes /V if no default exists.
pub fn reset_field(doc: &mut LopdfDoc, field_name: &str) -> Result<(), FormError> {
    let fields = get_form_fields(doc);
    let target = fields
        .into_iter()
        .find(|f| f.name == field_name)
        .ok_or_else(|| FormError::FieldNotFound(field_name.to_string()))?;

    if target.read_only {
        return Err(FormError::ReadOnlyField(field_name.to_string()));
    }

    let dict = doc.get_object_mut(target.object_id)?.as_dict_mut()?;
    if let Some(dv) = target.default_value {
        dict.set(
            b"V",
            Object::String(dv.as_bytes().to_vec(), StringFormat::Literal),
        );
    } else {
        dict.remove(b"V");
    }

    set_need_appearances(doc);
    Ok(())
}

/// Resets all editable fields to their defaults as a single all-or-nothing update.
/// Returns the number of fields reset. A failure leaves the original document untouched.
pub fn reset_all_fields(doc: &mut LopdfDoc) -> Result<usize, FormError> {
    let fields = get_form_fields(doc);
    if let Some(field) = fields.iter().find(|field| field.read_only) {
        return Err(FormError::ReadOnlyField(field.name.clone()));
    }

    let mut candidate = doc.clone();
    let mut count = 0;
    for field in &fields {
        reset_field(&mut candidate, &field.name)?;
        count += 1;
    }
    *doc = candidate;
    Ok(count)
}

fn set_need_appearances(doc: &mut LopdfDoc) {
    let root_id = match doc
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|root| root.as_reference().ok())
    {
        Some(id) => id,
        None => return,
    };

    let acro_form_id = doc
        .get_object(root_id)
        .ok()
        .and_then(|root| root.as_dict().ok())
        .and_then(|root| root.get(b"AcroForm").ok())
        .and_then(|acro_form| acro_form.as_reference().ok());

    if let Some(acro_form_id) = acro_form_id {
        if let Ok(Object::Dictionary(ref mut dict)) = doc.get_object_mut(acro_form_id) {
            dict.set(b"NeedAppearances", Object::Boolean(true));
        }
        return;
    }

    // AcroForm may legally be a direct catalog dictionary.
    if let Ok(Object::Dictionary(ref mut catalog)) = doc.get_object_mut(root_id) {
        if let Ok(Object::Dictionary(ref mut dict)) = catalog.get_mut(b"AcroForm") {
            dict.set(b"NeedAppearances", Object::Boolean(true));
        }
    }
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
    let mut candidate = doc.clone();
    let mut updated = 0;

    // Apply every requested value to a clone. Unknown/read-only fields or malformed
    // values abort the import without leaving a partially modified document.
    for (name, value) in map {
        set_field_value(&mut candidate, &name, &value)?;
        updated += 1;
    }

    *doc = candidate;
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::ObjectId;

    fn make_form_doc(fields: &[(&str, &str, bool)]) -> (LopdfDoc, Vec<ObjectId>) {
        let mut doc = LopdfDoc::with_version("1.7");
        let mut field_refs = Vec::new();
        let mut field_ids = Vec::new();

        for (name, value, read_only) in fields {
            let mut field = lopdf::Dictionary::new();
            field.set(
                b"T",
                Object::String(name.as_bytes().to_vec(), StringFormat::Literal),
            );
            field.set(b"FT", Object::Name(b"Tx".to_vec()));
            field.set(
                b"V",
                Object::String(value.as_bytes().to_vec(), StringFormat::Literal),
            );
            field.set(
                b"DV",
                Object::String(b"default".to_vec(), StringFormat::Literal),
            );
            field.set(b"Ff", Object::Integer(if *read_only { 1 } else { 0 }));
            let id = doc.add_object(field);
            field_ids.push(id);
            field_refs.push(Object::Reference(id));
        }

        let mut acro_form = lopdf::Dictionary::new();
        acro_form.set(b"Fields", Object::Array(field_refs));
        let acro_id = doc.add_object(acro_form);

        let mut catalog = lopdf::Dictionary::new();
        catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
        catalog.set(b"AcroForm", Object::Reference(acro_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set(b"Root", Object::Reference(catalog_id));

        (doc, field_ids)
    }

    #[test]
    fn read_only_fields_reject_updates() {
        let (mut doc, ids) = make_form_doc(&[("locked", "original", true)]);
        assert!(matches!(
            set_field_value(&mut doc, "locked", "changed"),
            Err(FormError::ReadOnlyField(name)) if name == "locked"
        ));
        let value = doc
            .get_object(ids[0])
            .unwrap()
            .as_dict()
            .unwrap()
            .get(b"V")
            .unwrap();
        assert_eq!(to_string_lossy(value).as_deref(), Some("original"));
    }

    #[test]
    fn reset_all_fields_is_atomic_when_a_field_is_read_only() {
        let (mut doc, ids) = make_form_doc(&[
            ("editable", "original", false),
            ("locked", "protected", true),
        ]);
        assert!(matches!(
            reset_all_fields(&mut doc),
            Err(FormError::ReadOnlyField(name)) if name == "locked"
        ));
        let first = doc
            .get_object(ids[0])
            .unwrap()
            .as_dict()
            .unwrap()
            .get(b"V")
            .unwrap();
        let second = doc
            .get_object(ids[1])
            .unwrap()
            .as_dict()
            .unwrap()
            .get(b"V")
            .unwrap();
        assert_eq!(to_string_lossy(first).as_deref(), Some("original"));
        assert_eq!(to_string_lossy(second).as_deref(), Some("protected"));
    }

    #[test]
    fn reset_field_restores_default_value() {
        let (mut doc, ids) = make_form_doc(&[("editable", "current", false)]);
        reset_field(&mut doc, "editable").unwrap();
        let value = doc
            .get_object(ids[0])
            .unwrap()
            .as_dict()
            .unwrap()
            .get(b"V")
            .unwrap();
        assert_eq!(to_string_lossy(value).as_deref(), Some("default"));
    }

    #[test]
    fn checkbox_values_use_pdf_names_and_update_appearance_state() {
        let mut doc = LopdfDoc::with_version("1.7");
        let mut normal = lopdf::Dictionary::new();
        normal.set(b"Off", Object::Null);
        normal.set(b"Yes", Object::Null);
        let mut appearance = lopdf::Dictionary::new();
        appearance.set(b"N", Object::Dictionary(normal));

        let mut field = lopdf::Dictionary::new();
        field.set(b"T", Object::String(b"subscribe".to_vec(), StringFormat::Literal));
        field.set(b"FT", Object::Name(b"Btn".to_vec()));
        field.set(b"V", Object::Name(b"Off".to_vec()));
        field.set(b"AS", Object::Name(b"Off".to_vec()));
        field.set(b"AP", Object::Dictionary(appearance));
        let field_id = doc.add_object(field);

        let mut acro_form = lopdf::Dictionary::new();
        acro_form.set(b"Fields", Object::Array(vec![Object::Reference(field_id)]));
        let acro_id = doc.add_object(acro_form);
        let mut catalog = lopdf::Dictionary::new();
        catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
        catalog.set(b"AcroForm", Object::Reference(acro_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set(b"Root", Object::Reference(catalog_id));

        set_field_value(&mut doc, "subscribe", "true").unwrap();
        let field = doc.get_object(field_id).unwrap().as_dict().unwrap();
        assert_eq!(field.get(b"V").unwrap().as_name().unwrap(), b"Yes");
        assert_eq!(field.get(b"AS").unwrap().as_name().unwrap(), b"Yes");

        set_field_value(&mut doc, "subscribe", "false").unwrap();
        let field = doc.get_object(field_id).unwrap().as_dict().unwrap();
        assert_eq!(field.get(b"V").unwrap().as_name().unwrap(), b"Off");
        assert_eq!(field.get(b"AS").unwrap().as_name().unwrap(), b"Off");
    }

    #[test]
    fn invalid_checkbox_value_does_not_partially_mutate_field() {
        let mut doc = LopdfDoc::with_version("1.7");
        let mut normal = lopdf::Dictionary::new();
        normal.set(b"Off", Object::Null);
        normal.set(b"Yes", Object::Null);
        let mut appearance = lopdf::Dictionary::new();
        appearance.set(b"N", Object::Dictionary(normal));
        let mut field = lopdf::Dictionary::new();
        field.set(b"T", Object::String(b"flag".to_vec(), StringFormat::Literal));
        field.set(b"FT", Object::Name(b"Btn".to_vec()));
        field.set(b"V", Object::Name(b"Off".to_vec()));
        field.set(b"AP", Object::Dictionary(appearance));
        let field_id = doc.add_object(field);
        let mut acro_form = lopdf::Dictionary::new();
        acro_form.set(b"Fields", Object::Array(vec![Object::Reference(field_id)]));
        let acro_id = doc.add_object(acro_form);
        let mut catalog = lopdf::Dictionary::new();
        catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
        catalog.set(b"AcroForm", Object::Reference(acro_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set(b"Root", Object::Reference(catalog_id));

        assert!(matches!(
            set_field_value(&mut doc, "flag", "not-a-button-state"),
            Err(FormError::InvalidButtonValue { field, .. }) if field == "flag"
        ));
        let field = doc.get_object(field_id).unwrap().as_dict().unwrap();
        assert_eq!(field.get(b"V").unwrap().as_name().unwrap(), b"Off");
    }

    #[test]
    fn nested_fields_use_qualified_names_and_inherited_read_only_flags() {
        let mut doc = LopdfDoc::with_version("1.7");

        let mut child = lopdf::Dictionary::new();
        child.set(
            b"T",
            Object::String(b"name".to_vec(), StringFormat::Literal),
        );
        child.set(
            b"V",
            Object::String(b"Sami".to_vec(), StringFormat::Literal),
        );
        let child_id = doc.add_object(child);

        let mut parent = lopdf::Dictionary::new();
        parent.set(
            b"T",
            Object::String(b"customer".to_vec(), StringFormat::Literal),
        );
        parent.set(b"FT", Object::Name(b"Tx".to_vec()));
        parent.set(b"Ff", Object::Integer(1));
        parent.set(b"Kids", Object::Array(vec![Object::Reference(child_id)]));
        let parent_id = doc.add_object(parent);

        let mut acro_form = lopdf::Dictionary::new();
        acro_form.set(b"Fields", Object::Array(vec![Object::Reference(parent_id)]));

        let mut catalog = lopdf::Dictionary::new();
        catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
        catalog.set(b"AcroForm", Object::Dictionary(acro_form));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set(b"Root", Object::Reference(catalog_id));

        let fields = get_form_fields(&doc);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "customer.name");
        assert_eq!(fields[0].kind, FieldKind::Text);
        assert!(fields[0].read_only);
        assert_eq!(fields[0].value.as_deref(), Some("Sami"));
        assert!(matches!(
            set_field_value(&mut doc, "customer.name", "Changed"),
            Err(FormError::ReadOnlyField(name)) if name == "customer.name"
        ));
    }

    #[test]
    fn form_data_import_is_atomic_when_any_field_is_invalid() {
        let (mut doc, ids) = make_form_doc(&[
            ("editable", "original", false),
            ("locked", "protected", true),
        ]);

        assert!(matches!(
            import_form_data_json(
                &mut doc,
                r#"{"editable":"changed","locked":"should-not-apply"}"#
            ),
            Err(FormError::ReadOnlyField(name)) if name == "locked"
        ));

        let first = doc
            .get_object(ids[0])
            .unwrap()
            .as_dict()
            .unwrap()
            .get(b"V")
            .unwrap();
        let second = doc
            .get_object(ids[1])
            .unwrap()
            .as_dict()
            .unwrap()
            .get(b"V")
            .unwrap();
        assert_eq!(to_string_lossy(first).as_deref(), Some("original"));
        assert_eq!(to_string_lossy(second).as_deref(), Some("protected"));
    }

    #[test]
    fn test_empty_form_fields() {
        let doc = LopdfDoc::with_version("1.7");
        let fields = get_form_fields(&doc);
        assert_eq!(fields.len(), 0);
        let json = export_form_data_json(&doc).unwrap();
        assert_eq!(json.trim(), "{}");
    }
}
