//! rustpdf-ui — Desktop UI components, theme engine, thumbnail customizer, viewer, organizer, and workbench app.

pub mod app;
pub mod home;
pub mod icons;
pub mod modals;
pub mod organizer;
pub mod search;
pub mod theme;
pub mod thumbnails;
pub mod tools_panel;
pub mod viewer;

pub use app::RustPdfApp;
pub use theme::{AccentPreset, AppTheme, ThemeMode};
pub use thumbnails::{ThumbnailConfig, ThumbnailPreset};
