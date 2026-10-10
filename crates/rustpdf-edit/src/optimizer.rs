//! rustpdf-edit/optimizer — Safe PDF compression, stream Flate encoding, and object graph pruning.

use lopdf::{Document as LopdfDoc, Object};
use crate::EditError;

/// Summary report of PDF compression and optimization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompressionResult {
    pub original_size: usize,
    pub compressed_size: usize,
    pub streams_compressed: usize,
    pub objects_pruned: usize,
    pub saved_bytes: usize,
}

impl CompressionResult {
    pub fn saved_ratio(&self) -> f32 {
        if self.original_size == 0 {
            0.0
        } else {
            self.saved_bytes as f32 / self.original_size as f32
        }
    }
}

/// Optimizes a PDF document by:
/// 1. Pruning unreachable and unreferenced indirect objects.
/// 2. Compressing eligible uncompressed streams using FlateDecode.
/// 3. Validating that the document remains well-formed and loadable.
pub fn compress_document(doc: &mut LopdfDoc) -> Result<CompressionResult, EditError> {
    // 1. Measure original serialized size
    let mut orig_buf = Vec::new();
    doc.save_to(&mut orig_buf)?;
    let original_size = orig_buf.len();

    // 2. Prune unreferenced objects from the object graph
    let pruned_list = doc.prune_objects();
    let objects_pruned = pruned_list.len();

    // 3. Compress uncompressed streams
    let mut streams_compressed = 0;
    for obj in doc.objects.values_mut() {
        if let Object::Stream(ref mut stream) = obj {
            if !stream.dict.has(b"Filter") {
                if let Ok(decomp) = stream.decompressed_content() {
                    // Try Flate compression via lopdf stream compression
                    stream.set_content(decomp);
                    if stream.compress().is_ok() {
                        streams_compressed += 1;
                    }
                }
            }
        }
    }

    // Also run overall document compression pass
    doc.compress();

    // 4. Measure optimized size
    let mut opt_buf = Vec::new();
    doc.save_to(&mut opt_buf)?;
    let compressed_size = opt_buf.len();

    // 5. Verify the optimized PDF is structurally valid
    LopdfDoc::load_mem(&opt_buf).map_err(EditError::Lopdf)?;

    let saved_bytes = original_size.saturating_sub(compressed_size);

    Ok(CompressionResult {
        original_size,
        compressed_size,
        streams_compressed,
        objects_pruned,
        saved_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_document_with_uncompressed_stream() {
        let mut doc = LopdfDoc::with_version("1.7");
        let stream_data = b"q 1 0 0 1 0 0 cm /F1 12 Tf (Hello Uncompressed PDF World) Tj Q".repeat(20);
        let stream = lopdf::Stream::new(lopdf::Dictionary::new(), stream_data);
        let id = doc.add_object(stream);
        let mut pages = lopdf::Dictionary::new();
        pages.set(b"Type", Object::Name(b"Pages".to_vec()));
        pages.set(b"Kids", Object::Array(vec![]));
        pages.set(b"Count", Object::Integer(0));
        let pages_id = doc.add_object(pages);
        
        let mut root = lopdf::Dictionary::new();
        root.set(b"Type", Object::Name(b"Catalog".to_vec()));
        root.set(b"Pages", Object::Reference(pages_id));
        root.set(b"TestStream", Object::Reference(id));
        let root_id = doc.add_object(root);
        doc.trailer.set(b"Root", Object::Reference(root_id));

        // Add an unreachable orphan object that should be pruned
        let orphan = Object::String(b"I am an orphaned string".to_vec(), lopdf::StringFormat::Literal);
        let _orphan_id = doc.add_object(orphan);

        let res = compress_document(&mut doc).unwrap();
        assert!(res.original_size > 0);
        assert!(res.compressed_size > 0);
        assert!(res.objects_pruned >= 1);
        assert!(res.streams_compressed >= 1);
    }
}
