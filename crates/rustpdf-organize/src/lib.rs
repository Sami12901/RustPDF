//! rustpdf-organize — Page tree operations: rotation, deletion, reordering, insertion, split, and merge.

use lopdf::{Dictionary, Document as LopdfDoc, Object, ObjectId, Stream};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrganizeError {
    #[error("document must retain at least one page")]
    CannotRemoveAllPages,
    #[error("page index {0} out of bounds (document has {1} pages)")]
    IndexOutOfBounds(usize, usize),
    #[error("invalid rotation {0}: degrees must be a multiple of 90")]
    InvalidRotation(i32),
    #[error("page order has a duplicate page index: {0}")]
    DuplicatePageIndex(usize),
    #[error("page order length mismatch: expected {expected} page indices, got {actual}")]
    InvalidPageOrderLength { expected: usize, actual: usize },
    #[error("invalid page range {start}..{end} for a document with {total} pages")]
    InvalidPageRange { start: usize, end: usize, total: usize },
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
    ensure_page_tree(doc)?;

    let mut page_ids: Vec<ObjectId> = doc.page_iter().collect();
    if index > page_ids.len() {
        return Err(OrganizeError::IndexOutOfBounds(index, page_ids.len()));
    }
    page_ids.insert(index, page_id);
    set_document_page_order(doc, &page_ids)
}

fn ensure_page_tree(doc: &mut LopdfDoc) -> Result<(ObjectId, ObjectId), OrganizeError> {
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
        let catalog = doc.get_object(catalog_id)?.as_dict()?.clone();
        if let Ok(pages_ref) = catalog.get(b"Pages").and_then(Object::as_reference) {
            pages_ref
        } else {
            let mut pages = Dictionary::new();
            pages.set(b"Type", Object::Name(b"Pages".to_vec()));
            pages.set(b"Count", Object::Integer(0));
            pages.set(b"Kids", Object::Array(Vec::new()));
            let id = doc.add_object(pages);
            doc.get_object_mut(catalog_id)?
                .as_dict_mut()?
                .set(b"Pages", Object::Reference(id));
            id
        }
    };

    Ok((catalog_id, pages_root_id))
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
    let current_pages = doc.get_pages();
    let total = current_pages.len();

    let mut page_nums = Vec::with_capacity(pages.len());
    for &idx in pages {
        if idx >= total {
            return Err(OrganizeError::IndexOutOfBounds(idx, total));
        }
        page_nums.push((idx + 1) as u32);
    }
    page_nums.sort_unstable();
    page_nums.dedup();

    if page_nums.len() >= total {
        return Err(OrganizeError::CannotRemoveAllPages);
    }
    if page_nums.is_empty() {
        return Ok(());
    }

    doc.delete_pages(&page_nums);
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
        return Err(OrganizeError::InvalidPageOrderLength {
            expected: total,
            actual: new_order.len(),
        });
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

/// Materializes inherited page attributes before flattening the page tree.
fn materialize_inherited_page_attributes(
    doc: &mut LopdfDoc,
    page_id: ObjectId,
    pages_root_id: ObjectId,
) -> Result<(), OrganizeError> {
    const INHERITABLE: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];

    let page_snapshot = doc.get_object(page_id)?.as_dict()?.clone();
    let mut inherited: Vec<(Vec<u8>, Object)> = Vec::new();

    for key in INHERITABLE {
        if page_snapshot.get(key).is_ok() {
            continue;
        }

        let mut parent_id = page_snapshot
            .get(b"Parent")
            .ok()
            .and_then(|obj| obj.as_reference().ok())
            .copied();

        while let Some(id) = parent_id {
            let parent = match doc.get_object(id).and_then(Object::as_dict) {
                Ok(dict) => dict,
                Err(_) => break,
            };

            if let Ok(value) = parent.get(key) {
                inherited.push((key.to_vec(), value.clone()));
                break;
            }

            parent_id = parent
                .get(b"Parent")
                .ok()
                .and_then(|obj| obj.as_reference().ok())
                .copied();
        }
    }

    let page = doc.get_object_mut(page_id)?.as_dict_mut()?;
    for (key, value) in inherited {
        page.set(key, value);
    }
    page.set(b"Parent", Object::Reference(pages_root_id));
    Ok(())
}

fn set_document_page_order(
    doc: &mut LopdfDoc,
    page_ids: &[ObjectId],
) -> Result<(), OrganizeError> {
    let (_, pages_root_id) = ensure_page_tree(doc)?;

    // Pages may inherit MediaBox, CropBox, Resources, and Rotate from intermediate
    // /Pages nodes. Preserve those values before rebuilding a flat, ordered tree.
    for &page_id in page_ids {
        materialize_inherited_page_attributes(doc, page_id, pages_root_id)?;
    }

    let pages_dict = doc.get_object_mut(pages_root_id)?.as_dict_mut()?;
    let kid_refs: Vec<Object> = page_ids.iter().map(|&id| Object::Reference(id)).collect();
    pages_dict.set(b"Kids", Object::Array(kid_refs));
    pages_dict.set(b"Count", Object::Integer(page_ids.len() as i64));

    Ok(())
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
        SplitRule::Ranges(ranges) => {
            for range in &ranges {
                if range.start >= range.end || range.end > total_pages {
                    return Err(OrganizeError::InvalidPageRange {
                        start: range.start,
                        end: range.end,
                        total: total_pages,
                    });
                }
            }
            ranges
        },
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
