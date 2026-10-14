//! rustpdf-organize — Page tree operations: rotation, deletion, reordering, insertion, split, and merge.

use lopdf::{Dictionary, Document as LopdfDoc, Object, ObjectId, Stream};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrganizeError {
    #[error("document must retain at least one page")]
    CannotRemoveAllPages,
    #[error("page index {0} out of bounds (document has {1} pages)")]
    IndexOutOfBounds(usize, usize),
    #[error("page index {0} appears more than once")]
    DuplicatePageIndex(usize),
    #[error("at least one page must be selected for extraction")]
    EmptyPageSelection,
    #[error("invalid rotation {0}: degrees must be a multiple of 90")]
    InvalidRotation(i32),
    #[error("lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("combine error: {0}")]
    CombineFailed(String),
}

/// Helper function to insert a page object into a document's Pages tree.
pub fn insert_page_into_document(
    doc: &mut LopdfDoc,
    index: usize,
    page_id: ObjectId,
) -> Result<(), OrganizeError> {
    let catalog_id = if let Ok(root_ref) = doc.trailer.get(b"Root").and_then(Object::as_reference) {
        root_ref
    } else {
        let mut catalog = Dictionary::new();
        catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
        let id = doc.add_object(catalog);
        doc.trailer.set(b"Root", Object::Reference(id));
        id
    };

    let pages_root_id = {
        let catalog = doc.get_object(catalog_id)?.as_dict()?;
        if let Ok(pages_ref) = catalog.get(b"Pages").and_then(Object::as_reference) {
            pages_ref
        } else {
            let mut pages_dict = Dictionary::new();
            pages_dict.set(b"Type", Object::Name(b"Pages".to_vec()));
            pages_dict.set(b"Count", Object::Integer(0));
            pages_dict.set(b"Kids", Object::Array(Vec::new()));
            let pages_id = doc.add_object(pages_dict);
            doc.get_object_mut(catalog_id)?
                .as_dict_mut()?
                .set(b"Pages", Object::Reference(pages_id));
            pages_id
        }
    };

    let mut page_ids: Vec<ObjectId> = doc.page_iter().collect();
    if index > page_ids.len() {
        return Err(OrganizeError::IndexOutOfBounds(index, page_ids.len()));
    }

    // Preserve attributes inherited from nested /Pages nodes before flattening
    // the page tree to a single ordered root Kids array.
    for &existing_id in &page_ids {
        copy_inherited_page_attributes(doc, existing_id)?;
    }
    copy_inherited_page_attributes(doc, page_id)?;

    page_ids.insert(index, page_id);
    set_document_page_order_at_root(doc, pages_root_id, &page_ids)
}

/// Copies inherited page attributes onto a leaf page before its /Parent changes.
fn copy_inherited_page_attributes(
    doc: &mut LopdfDoc,
    page_id: ObjectId,
) -> Result<(), OrganizeError> {
    const INHERITABLE_KEYS: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];

    let page_dict = doc.get_object(page_id)?.as_dict()?;
    let parent_id = page_dict
        .get(b"Parent")
        .ok()
        .and_then(|object| object.as_reference().ok());
    let mut inherited = Vec::new();

    for key in INHERITABLE_KEYS {
        if page_dict.get(key).is_ok() {
            continue;
        }

        let mut cursor = parent_id;
        let mut visited = std::collections::HashSet::new();
        let mut found = None;
        while let Some(id) = cursor {
            if !visited.insert(id) {
                break;
            }
            let parent = doc.get_object(id)?.as_dict()?;
            if let Ok(value) = parent.get(key) {
                found = Some(value.clone());
                break;
            }
            cursor = parent
                .get(b"Parent")
                .ok()
                .and_then(|object| object.as_reference().ok());
        }

        if let Some(value) = found {
            inherited.push((key, value));
        }
    }

    if !inherited.is_empty() {
        let dict = doc.get_object_mut(page_id)?.as_dict_mut()?;
        for (key, value) in inherited {
            dict.set(key, value);
        }
    }

    Ok(())
}

/// Rotates selected pages by degrees (clockwise positive, counter-clockwise negative; multiple of 90).
pub fn rotate_pages(
    doc: &mut LopdfDoc,
    pages: &[usize],
    degrees: i32,
) -> Result<(), OrganizeError> {
    if degrees % 90 != 0 {
        return Err(OrganizeError::InvalidRotation(degrees));
    }

    let page_map = doc.get_pages();
    let total_pages = page_map.len();

    for &page_idx in pages {
        let page_num = (page_idx + 1) as u32;
        let &obj_id = page_map
            .get(&page_num)
            .ok_or(OrganizeError::IndexOutOfBounds(page_idx, total_pages))?;

        if let Ok(Object::Dictionary(ref mut dict)) = doc.get_object_mut(obj_id) {
            let current = dict
                .get(b"Rotate")
                .ok()
                .and_then(|o| o.as_i64().ok())
                .unwrap_or(0) as i32;

            let mut new_rot = (current + degrees) % 360;
            if new_rot < 0 {
                new_rot += 360;
            }

            dict.set(b"Rotate", Object::Integer(new_rot as i64));
        }
    }

    Ok(())
}

/// Deletes specified pages from the document. Fails if all pages would be removed.
pub fn delete_pages(doc: &mut LopdfDoc, pages: &[usize]) -> Result<(), OrganizeError> {
    let total = doc.get_pages().len();
    let mut unique_pages = std::collections::BTreeSet::new();

    for &page_idx in pages {
        if page_idx >= total {
            return Err(OrganizeError::IndexOutOfBounds(page_idx, total));
        }
        unique_pages.insert(page_idx);
    }

    if unique_pages.len() >= total {
        return Err(OrganizeError::CannotRemoveAllPages);
    }

    // lopdf expects 1-based page numbers. Deduplicate repeated UI selections.
    let page_nums: Vec<u32> = unique_pages
        .into_iter()
        .map(|idx| (idx + 1) as u32)
        .collect();

    if !page_nums.is_empty() {
        doc.delete_pages(&page_nums);
    }
    Ok(())
}

/// Inserts a blank page with given width and height at the specified index.
pub fn insert_blank_page(
    doc: &mut LopdfDoc,
    index: usize,
    width: f32,
    height: f32,
) -> Result<(), OrganizeError> {
    // Create an empty content stream
    let content_id = doc.add_object(Stream::new(Dictionary::new(), Vec::new()));

    // Create empty resources dictionary
    let resources_id = doc.add_object(Dictionary::new());

    // Create Page dictionary
    let mut page_dict = Dictionary::new();
    page_dict.set(b"Type", Object::Name(b"Page".to_vec()));
    page_dict.set(
        b"MediaBox",
        Object::Array(vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(width),
            Object::Real(height),
        ]),
    );
    page_dict.set(b"Contents", Object::Reference(content_id));
    page_dict.set(b"Resources", Object::Reference(resources_id));

    let page_id = doc.add_object(page_dict);
    insert_page_into_document(doc, index, page_id)
}

/// Duplicates the page at the given index, placing the copy directly after it.
pub fn duplicate_page(doc: &mut LopdfDoc, index: usize) -> Result<(), OrganizeError> {
    let page_map = doc.get_pages();
    let total = page_map.len();
    let page_num = (index + 1) as u32;

    let &source_id = page_map
        .get(&page_num)
        .ok_or(OrganizeError::IndexOutOfBounds(index, total))?;

    let source_dict = doc.get_object(source_id)?.as_dict()?.clone();
    let copy_id = doc.add_object(source_dict);

    insert_page_into_document(doc, index + 1, copy_id)
}

/// Moves a page from one index to another (0-indexed).
pub fn move_page(
    doc: &mut LopdfDoc,
    from_index: usize,
    to_index: usize,
) -> Result<(), OrganizeError> {
    let pages: Vec<ObjectId> = doc.page_iter().collect();
    let total = pages.len();
    if from_index >= total {
        return Err(OrganizeError::IndexOutOfBounds(from_index, total));
    }
    if to_index >= total {
        return Err(OrganizeError::IndexOutOfBounds(to_index, total));
    }
    if from_index == to_index {
        return Ok(());
    }

    let mut new_pages = pages;
    let page_id = new_pages.remove(from_index);
    new_pages.insert(to_index, page_id);

    set_document_page_order(doc, &new_pages)
}

/// Reorders pages according to a sequence of page indices (e.g. `[2, 0, 1]`).
pub fn reorder_pages(doc: &mut LopdfDoc, new_order: &[usize]) -> Result<(), OrganizeError> {
    let pages: Vec<ObjectId> = doc.page_iter().collect();
    let total = pages.len();
    if new_order.len() != total {
        return Err(OrganizeError::IndexOutOfBounds(new_order.len(), total));
    }

    let mut seen = std::collections::HashSet::with_capacity(total);
    let mut reordered = Vec::with_capacity(total);
    for &idx in new_order {
        if idx >= total {
            return Err(OrganizeError::IndexOutOfBounds(idx, total));
        }
        if !seen.insert(idx) {
            return Err(OrganizeError::DuplicatePageIndex(idx));
        }
        reordered.push(pages[idx]);
    }

    set_document_page_order(doc, &reordered)
}

fn set_document_page_order(doc: &mut LopdfDoc, page_ids: &[ObjectId]) -> Result<(), OrganizeError> {
    let catalog_id = doc
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .map_err(|_| OrganizeError::CombineFailed("Missing catalog".to_string()))?;
    let pages_root_id = doc
        .get_object(catalog_id)?
        .as_dict()?
        .get(b"Pages")
        .and_then(Object::as_reference)
        .map_err(|_| OrganizeError::CombineFailed("Missing pages root".to_string()))?;

    for &page_id in page_ids {
        copy_inherited_page_attributes(doc, page_id)?;
    }
    set_document_page_order_at_root(doc, pages_root_id, page_ids)
}

fn set_document_page_order_at_root(
    doc: &mut LopdfDoc,
    pages_root_id: ObjectId,
    page_ids: &[ObjectId],
) -> Result<(), OrganizeError> {
    for &page_id in page_ids {
        doc.get_object_mut(page_id)?
            .as_dict_mut()?
            .set(b"Parent", Object::Reference(pages_root_id));
    }

    let pages_dict = doc.get_object_mut(pages_root_id)?.as_dict_mut()?;
    let kid_refs: Vec<Object> = page_ids.iter().map(|&id| Object::Reference(id)).collect();
    pages_dict.set(b"Kids", Object::Array(kid_refs));
    pages_dict.set(b"Count", Object::Integer(page_ids.len() as i64));

    Ok(())
}

/// Extracts the requested zero-based pages into a separate standalone PDF.
///
/// Pages are emitted in their original document order. Repeated selections are
/// deduplicated; any out-of-range index rejects the operation without modifying
/// the caller's original bytes.
pub fn extract_pages(doc_bytes: &[u8], page_indices: &[usize]) -> Result<Vec<u8>, OrganizeError> {
    if page_indices.is_empty() {
        return Err(OrganizeError::EmptyPageSelection);
    }

    let mut doc = LopdfDoc::load_mem(doc_bytes)?;
    let total = doc.get_pages().len();
    let mut selected = std::collections::BTreeSet::new();

    for &index in page_indices {
        if index >= total {
            return Err(OrganizeError::IndexOutOfBounds(index, total));
        }
        selected.insert(index);
    }

    let to_delete: Vec<u32> = (0..total)
        .filter(|index| !selected.contains(index))
        .map(|index| (index + 1) as u32)
        .collect();

    if !to_delete.is_empty() {
        doc.delete_pages(&to_delete);
    }

    if doc.get_pages().len() != selected.len() {
        return Err(OrganizeError::CombineFailed(
            "Extracted page count does not match the requested selection".to_string(),
        ));
    }

    let mut output = Vec::new();
    doc.save_to(&mut output)?;
    Ok(output)
}

#[cfg(test)]
mod extraction_tests {
    use super::*;

    #[test]
    fn extract_pages_preserves_document_order_and_deduplicates_selection() {
        let mut original = LopdfDoc::with_version("1.7");
        for i in 0..4 {
            insert_blank_page(&mut original, i, 612.0 + i as f32, 792.0).unwrap();
        }
        let mut source = Vec::new();
        original.save_to(&mut source).unwrap();

        let extracted = extract_pages(&source, &[3, 1, 1]).unwrap();
        let extracted_doc = LopdfDoc::load_mem(&extracted).unwrap();
        let pages = extracted_doc.get_pages();
        assert_eq!(pages.len(), 2);

        let widths: Vec<f32> = pages
            .values()
            .map(|id| {
                let page = extracted_doc.get_object(*id).unwrap().as_dict().unwrap();
                page.get(b"MediaBox").unwrap().as_array().unwrap()[2]
                    .as_float()
                    .unwrap()
            })
            .collect();
        assert_eq!(widths, vec![613.0, 615.0]);
    }

    #[test]
    fn extract_pages_rejects_empty_or_out_of_range_selection() {
        let mut doc = LopdfDoc::with_version("1.7");
        insert_blank_page(&mut doc, 0, 612.0, 792.0).unwrap();
        let mut source = Vec::new();
        doc.save_to(&mut source).unwrap();

        assert!(matches!(
            extract_pages(&source, &[]),
            Err(OrganizeError::EmptyPageSelection)
        ));
        assert!(matches!(
            extract_pages(&source, &[1]),
            Err(OrganizeError::IndexOutOfBounds(1, 1))
        ));
        assert_eq!(LopdfDoc::load_mem(&source).unwrap().get_pages().len(), 1);
    }
}

/// Merges multiple PDF document byte buffers into a single combined PDF.
pub fn combine_documents(inputs: &[Vec<u8>]) -> Result<Vec<u8>, OrganizeError> {
    if inputs.is_empty() {
        return Err(OrganizeError::CombineFailed(
            "No input documents provided".to_string(),
        ));
    }

    let mut target = LopdfDoc::load_mem(&inputs[0])?;

    for (idx, input_bytes) in inputs[1..].iter().enumerate() {
        let source = LopdfDoc::load_mem(input_bytes).map_err(|e| {
            OrganizeError::CombineFailed(format!("Failed to load doc #{}: {e}", idx + 2))
        })?;

        let target_page_count = target.get_pages().len();

        // Renumber objects in source document so they don't collide with target
        let mut renumbered_source = source.clone();
        let max_id = target.max_id;
        renumbered_source.renumber_objects_with(max_id + 1);

        for (&id, obj) in &renumbered_source.objects {
            target.objects.entry(id).or_insert_with(|| obj.clone());
        }

        let source_pages: Vec<ObjectId> = renumbered_source.page_iter().collect();
        for (i, page_id) in source_pages.into_iter().enumerate() {
            insert_page_into_document(&mut target, target_page_count + i, page_id)?;
        }
    }

    let mut out = Vec::new();
    target.save_to(&mut out)?;
    Ok(out)
}

/// Specification for splitting a document.
#[derive(Clone, Debug, PartialEq)]
pub enum SplitRule {
    /// Every N pages per output document.
    EveryN(usize),
    /// Page ranges, e.g. [(0..3), (3..7)].
    Ranges(Vec<std::ops::Range<usize>>),
    /// Single page per file.
    SinglePage,
}

/// Splits a document into multiple document buffers based on the SplitRule.
pub fn split_document(doc_bytes: &[u8], rule: SplitRule) -> Result<Vec<Vec<u8>>, OrganizeError> {
    let doc = LopdfDoc::load_mem(doc_bytes)?;
    let total_pages = doc.get_pages().len();

    let ranges = match rule {
        SplitRule::SinglePage => (0..total_pages).map(|i| i..i + 1).collect(),
        SplitRule::EveryN(n) => {
            let chunk_size = n.max(1);
            let mut r = Vec::new();
            let mut start = 0;
            while start < total_pages {
                let end = (start + chunk_size).min(total_pages);
                r.push(start..end);
                start = end;
            }
            r
        }
        SplitRule::Ranges(r) => r,
    };

    let mut outputs = Vec::with_capacity(ranges.len());

    for range in ranges {
        if range.start >= total_pages {
            continue;
        }
        let end = range.end.min(total_pages);

        // Keep only pages within the range by deleting pages outside it
        let mut sub_doc = doc.clone();
        let pages_to_delete: Vec<u32> = (0..total_pages)
            .filter(|&i| i < range.start || i >= end)
            .map(|i| (i + 1) as u32)
            .collect();

        if pages_to_delete.len() < total_pages {
            sub_doc.delete_pages(&pages_to_delete);
            let mut buf = Vec::new();
            sub_doc.save_to(&mut buf)?;
            outputs.push(buf);
        }
    }

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_and_delete() {
        let mut doc = LopdfDoc::with_version("1.7");
        // insert blank pages
        insert_blank_page(&mut doc, 0, 612.0, 792.0).unwrap();
        insert_blank_page(&mut doc, 1, 612.0, 792.0).unwrap();

        assert_eq!(doc.get_pages().len(), 2);

        // Rotate page 0 by 90 degrees
        rotate_pages(&mut doc, &[0], 90).unwrap();

        // Delete page 1
        delete_pages(&mut doc, &[1]).unwrap();
        assert_eq!(doc.get_pages().len(), 1);

        // Trying to delete last page must fail
        assert!(delete_pages(&mut doc, &[0]).is_err());
    }

    #[test]
    fn insertion_flattens_nested_page_tree_without_losing_inherited_attributes() {
        let mut doc = LopdfDoc::with_version("1.7");
        let resources_id = doc.add_object(Dictionary::new());

        let top_pages_id = doc.new_object_id();
        let nested_pages_id = doc.new_object_id();

        let mut page_a = Dictionary::new();
        page_a.set(b"Type", Object::Name(b"Page".to_vec()));
        page_a.set(b"Parent", Object::Reference(nested_pages_id));
        let page_a_id = doc.add_object(page_a);

        let mut page_b = Dictionary::new();
        page_b.set(b"Type", Object::Name(b"Page".to_vec()));
        page_b.set(b"Parent", Object::Reference(nested_pages_id));
        let page_b_id = doc.add_object(page_b);

        let mut nested_pages = Dictionary::new();
        nested_pages.set(b"Type", Object::Name(b"Pages".to_vec()));
        nested_pages.set(b"Parent", Object::Reference(top_pages_id));
        nested_pages.set(b"Count", Object::Integer(2));
        nested_pages.set(
            b"Kids",
            Object::Array(vec![
                Object::Reference(page_a_id),
                Object::Reference(page_b_id),
            ]),
        );
        nested_pages.set(
            b"MediaBox",
            Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(612.0),
                Object::Real(792.0),
            ]),
        );
        nested_pages.set(b"Resources", Object::Reference(resources_id));
        doc.set_object(nested_pages_id, nested_pages);

        let mut top_pages = Dictionary::new();
        top_pages.set(b"Type", Object::Name(b"Pages".to_vec()));
        top_pages.set(b"Count", Object::Integer(2));
        top_pages.set(
            b"Kids",
            Object::Array(vec![Object::Reference(nested_pages_id)]),
        );
        doc.set_object(top_pages_id, top_pages);

        let mut catalog = Dictionary::new();
        catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
        catalog.set(b"Pages", Object::Reference(top_pages_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set(b"Root", Object::Reference(catalog_id));

        insert_blank_page(&mut doc, 1, 555.0, 777.0).unwrap();

        let pages = doc.get_pages();
        assert_eq!(pages.len(), 3);
        let widths: Vec<f32> = pages
            .values()
            .map(|id| {
                let page = doc.get_object(*id).unwrap().as_dict().unwrap();
                assert!(page.get(b"Resources").is_ok());
                page.get(b"MediaBox").unwrap().as_array().unwrap()[2]
                    .as_float()
                    .unwrap()
            })
            .collect();
        assert_eq!(widths, vec![612.0, 555.0, 612.0]);

        let root = doc.get_object(top_pages_id).unwrap().as_dict().unwrap();
        let kids = root.get(b"Kids").unwrap().as_array().unwrap();
        assert_eq!(kids.len(), 3);
        assert!(kids.iter().all(|child| child.as_reference().is_ok()));
    }

    #[test]
    fn test_delete_pages_validates_indices_and_deduplicates() {
        let mut doc = LopdfDoc::with_version("1.7");
        for i in 0..3 {
            insert_blank_page(&mut doc, i, 612.0, 792.0).unwrap();
        }

        delete_pages(&mut doc, &[1, 1]).unwrap();
        assert_eq!(doc.get_pages().len(), 2);
        assert!(matches!(
            delete_pages(&mut doc, &[2]),
            Err(OrganizeError::IndexOutOfBounds(2, 2))
        ));
        assert!(matches!(
            delete_pages(&mut doc, &[0, 0, 2]),
            Err(OrganizeError::IndexOutOfBounds(2, 2))
        ));
    }

    #[test]
    fn test_reorder_pages_rejects_duplicate_indices() {
        let mut doc = LopdfDoc::with_version("1.7");
        for i in 0..3 {
            insert_blank_page(&mut doc, i, 612.0, 792.0).unwrap();
        }

        assert!(matches!(
            reorder_pages(&mut doc, &[0, 0, 2]),
            Err(OrganizeError::DuplicatePageIndex(0))
        ));
        assert_eq!(doc.get_pages().len(), 3);
    }

    #[test]
    fn test_move_and_reorder_pages() {
        let mut doc = LopdfDoc::with_version("1.7");
        insert_blank_page(&mut doc, 0, 612.0, 792.0).unwrap();
        insert_blank_page(&mut doc, 1, 612.0, 792.0).unwrap();
        insert_blank_page(&mut doc, 2, 612.0, 792.0).unwrap();

        let initial_order: Vec<ObjectId> = doc.page_iter().collect();
        assert_eq!(initial_order.len(), 3);

        // Move page 0 to index 2
        move_page(&mut doc, 0, 2).unwrap();
        let moved_order: Vec<ObjectId> = doc.page_iter().collect();
        assert_eq!(
            moved_order,
            vec![initial_order[1], initial_order[2], initial_order[0]]
        );

        // Reorder back: [2, 0, 1]
        reorder_pages(&mut doc, &[2, 0, 1]).unwrap();
        let reordered: Vec<ObjectId> = doc.page_iter().collect();
        assert_eq!(reordered, initial_order);
    }
}
