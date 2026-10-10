//! rustpdf-ui/thumbnails — Thumbnail configuration engine, presets, and persistence.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThumbnailPreset {
    Compact,
    Balanced,
    Large,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThumbnailConfig {
    pub preset: ThumbnailPreset,
    /// Thumbnail width in points/pixels (80.0 to 320.0)
    pub size: f32,
    /// Grid spacing / density factor (0.5 = compact, 1.0 = normal, 1.5 = spacious)
    pub density: f32,
    /// Sidebar panel width in points (120.0 to 360.0)
    pub sidebar_width: f32,
    /// Corner radius for thumbnail cards (0.0 to 16.0)
    pub corner_radius: f32,
    /// Display subtle drop shadow under thumbnail cards
    pub show_shadow: bool,
    /// Display page index / label badge below thumbnail
    pub show_page_labels: bool,
    /// Display physical page dimensions (e.g. 8.5 x 11 in)
    pub show_dimensions: bool,
    /// Border thickness on selection
    pub border_width: f32,
}

impl Default for ThumbnailConfig {
    fn default() -> Self {
        Self::preset_balanced()
    }
}

impl ThumbnailConfig {
    pub fn preset_compact() -> Self {
        Self {
            preset: ThumbnailPreset::Compact,
            size: 96.0,
            density: 0.7,
            sidebar_width: 140.0,
            corner_radius: 2.0,
            show_shadow: false,
            show_page_labels: true,
            show_dimensions: false,
            border_width: 2.0,
        }
    }

    pub fn preset_balanced() -> Self {
        Self {
            preset: ThumbnailPreset::Balanced,
            size: 140.0,
            density: 1.0,
            sidebar_width: 190.0,
            corner_radius: 6.0,
            show_shadow: true,
            show_page_labels: true,
            show_dimensions: true,
            border_width: 2.5,
        }
    }

    pub fn preset_large() -> Self {
        Self {
            preset: ThumbnailPreset::Large,
            size: 220.0,
            density: 1.3,
            sidebar_width: 260.0,
            corner_radius: 8.0,
            show_shadow: true,
            show_page_labels: true,
            show_dimensions: true,
            border_width: 3.0,
        }
    }

    pub fn apply_preset(&mut self, preset: ThumbnailPreset) {
        match preset {
            ThumbnailPreset::Compact => *self = Self::preset_compact(),
            ThumbnailPreset::Balanced => *self = Self::preset_balanced(),
            ThumbnailPreset::Large => *self = Self::preset_large(),
            ThumbnailPreset::Custom => self.preset = ThumbnailPreset::Custom,
        }
    }

    pub fn config_path() -> PathBuf {
        let mut p = if let Some(base) = std::env::var_os("APPDATA") {
            PathBuf::from(base).join("RustPdf")
        } else if let Some(base) = std::env::var_os("HOME") {
            PathBuf::from(base).join(".config").join("rustpdf")
        } else {
            PathBuf::from(".")
        };
        p.push("rustpdf_thumbnails.json");
        p
    }

    pub fn load_persisted() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save_persisted(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, data)?;
        Ok(())
    }
}
