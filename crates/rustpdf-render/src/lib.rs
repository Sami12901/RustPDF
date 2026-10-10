//! rustpdf-render — Page rasterization and text extraction pipeline.

pub mod pool;
pub mod text;

pub use pool::{RenderPool, RenderRequest, RenderedPage, effective_scale, device_pixels};
pub use text::{PageText, TextGlyph};
