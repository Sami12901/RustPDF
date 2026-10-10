//! rustpdf-render — Page rasterization and text extraction pipeline.

pub mod pool;
pub mod text;

pub use pool::{device_pixels, effective_scale, RenderPool, RenderRequest, RenderedPage};
pub use text::{PageText, TextGlyph};
