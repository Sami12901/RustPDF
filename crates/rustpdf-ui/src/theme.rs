//! rustpdf-ui/theme — Appearance settings, theme customization engine, and semantic tokens.

use egui::{Color32, Context, Stroke, Visuals};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccentPreset {
    RustOrange, // #E05D26 (Primary brand)
    Blue,       // #2A7BE4
    Green,      // #10B981
    Violet,     // #8B5CF6
    Teal,       // #14B8A6
    Custom,
}

impl AccentPreset {
    pub fn default_color(self) -> Color32 {
        match self {
            Self::RustOrange => Color32::from_rgb(224, 93, 38),
            Self::Blue => Color32::from_rgb(42, 123, 228),
            Self::Green => Color32::from_rgb(16, 185, 129),
            Self::Violet => Color32::from_rgb(139, 92, 246),
            Self::Teal => Color32::from_rgb(20, 184, 166),
            Self::Custom => Color32::from_rgb(224, 93, 38),
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::RustOrange => "Rust Orange",
            Self::Blue => "Ocean Blue",
            Self::Green => "Emerald Green",
            Self::Violet => "Royal Violet",
            Self::Teal => "Nordic Teal",
            Self::Custom => "Custom Accent",
        }
    }
}

/// Fully customizable semantic color tokens.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppTheme {
    pub mode: ThemeMode,
    pub accent_preset: AccentPreset,
    pub custom_accent: [u8; 3],

    // Dark Palette Tokens
    pub dark_bg: [u8; 3],
    pub dark_panel: [u8; 3],
    pub dark_surface: [u8; 3],
    pub dark_border: [u8; 3],
    pub dark_text_primary: [u8; 3],
    pub dark_text_secondary: [u8; 3],

    // Light Palette Tokens
    pub light_bg: [u8; 3],
    pub light_panel: [u8; 3],
    pub light_surface: [u8; 3],
    pub light_border: [u8; 3],
    pub light_text_primary: [u8; 3],
    pub light_text_secondary: [u8; 3],
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Dark,
            accent_preset: AccentPreset::RustOrange,
            custom_accent: [224, 93, 38],

            // Dark: Sleek, high-contrast, modern neutral surfaces
            dark_bg: [18, 19, 22],
            dark_panel: [24, 25, 29],
            dark_surface: [32, 34, 40],
            dark_border: [48, 51, 60],
            dark_text_primary: [240, 242, 245],
            dark_text_secondary: [156, 163, 175],

            // Light: Warm editorial, clean borders, non-glare surfaces
            light_bg: [245, 246, 248],
            light_panel: [255, 255, 255],
            light_surface: [238, 240, 244],
            light_border: [218, 222, 229],
            light_text_primary: [17, 24, 39],
            light_text_secondary: [107, 114, 128],
        }
    }
}

impl AppTheme {
    pub fn active_accent(&self) -> Color32 {
        if self.accent_preset == AccentPreset::Custom {
            Color32::from_rgb(
                self.custom_accent[0],
                self.custom_accent[1],
                self.custom_accent[2],
            )
        } else {
            self.accent_preset.default_color()
        }
    }

    /// Resolves this preference to a concrete theme.
    ///
    /// If the OS theme cannot be detected while System is selected, dark mode
    /// is used as a stable fallback.
    pub fn is_dark(&self) -> bool {
        self.is_dark_for(None)
    }

    /// Resolves this preference using the system theme reported by egui.
    pub fn is_dark_for(&self, system_theme: Option<egui::Theme>) -> bool {
        match self.mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => system_theme.unwrap_or(egui::Theme::Dark) == egui::Theme::Dark,
        }
    }

    pub fn bg_color(&self) -> Color32 {
        self.bg_color_for(None)
    }

    pub fn bg_color_for(&self, system_theme: Option<egui::Theme>) -> Color32 {
        let palette = if self.is_dark_for(system_theme) {
            self.dark_bg
        } else {
            self.light_bg
        };
        Color32::from_rgb(palette[0], palette[1], palette[2])
    }

    pub fn panel_color(&self) -> Color32 {
        self.panel_color_for(None)
    }

    pub fn panel_color_for(&self, system_theme: Option<egui::Theme>) -> Color32 {
        let palette = if self.is_dark_for(system_theme) {
            self.dark_panel
        } else {
            self.light_panel
        };
        Color32::from_rgb(palette[0], palette[1], palette[2])
    }

    pub fn surface_color(&self) -> Color32 {
        self.surface_color_for(None)
    }

    pub fn surface_color_for(&self, system_theme: Option<egui::Theme>) -> Color32 {
        let palette = if self.is_dark_for(system_theme) {
            self.dark_surface
        } else {
            self.light_surface
        };
        Color32::from_rgb(palette[0], palette[1], palette[2])
    }

    pub fn border_color(&self) -> Color32 {
        self.border_color_for(None)
    }

    pub fn border_color_for(&self, system_theme: Option<egui::Theme>) -> Color32 {
        let palette = if self.is_dark_for(system_theme) {
            self.dark_border
        } else {
            self.light_border
        };
        Color32::from_rgb(palette[0], palette[1], palette[2])
    }

    pub fn text_primary(&self) -> Color32 {
        self.text_primary_for(None)
    }

    pub fn text_primary_for(&self, system_theme: Option<egui::Theme>) -> Color32 {
        let palette = if self.is_dark_for(system_theme) {
            self.dark_text_primary
        } else {
            self.light_text_primary
        };
        Color32::from_rgb(palette[0], palette[1], palette[2])
    }

    pub fn text_secondary(&self) -> Color32 {
        self.text_secondary_for(None)
    }

    pub fn text_secondary_for(&self, system_theme: Option<egui::Theme>) -> Color32 {
        let palette = if self.is_dark_for(system_theme) {
            self.dark_text_secondary
        } else {
            self.light_text_secondary
        };
        Color32::from_rgb(palette[0], palette[1], palette[2])
    }

    /// Injects theme into egui Visuals.
    pub fn apply_to_egui(&self, ctx: &Context) {
        let system_theme = ctx.system_theme();
        let is_dark = self.is_dark_for(system_theme);
        let mut visuals = if is_dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        let accent = self.active_accent();
        let bg = self.bg_color_for(system_theme);
        let panel = self.panel_color_for(system_theme);
        let surface = self.surface_color_for(system_theme);
        let border = self.border_color_for(system_theme);
        let text_p = self.text_primary_for(system_theme);
        let text_s = self.text_secondary_for(system_theme);

        visuals.window_fill = panel;
        visuals.panel_fill = bg;
        visuals.faint_bg_color = surface;
        visuals.extreme_bg_color = if is_dark {
            Color32::from_rgb(14, 15, 17)
        } else {
            Color32::from_rgb(255, 255, 255)
        };

        visuals.window_stroke = Stroke::new(1.0, border);
        visuals.widgets.noninteractive.bg_fill = panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_p);

        visuals.widgets.inactive.bg_fill = surface;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_s);

        visuals.widgets.hovered.bg_fill = if is_dark {
            Color32::from_rgb(44, 48, 58)
        } else {
            Color32::from_rgb(228, 232, 238)
        };
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text_p);

        visuals.widgets.active.bg_fill = accent;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(
            accent.r(),
            accent.g(),
            accent.b(),
            if is_dark { 100 } else { 80 },
        );
        visuals.selection.stroke = Stroke::new(1.0, accent);

        ctx.set_visuals(visuals);
    }

    pub fn config_path() -> PathBuf {
        let mut p = dirs_fallback();
        p.push("rustpdf_theme.json");
        p
    }

    pub fn load_persisted() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(theme) = serde_json::from_str(&data) {
                return theme;
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

fn dirs_fallback() -> PathBuf {
    if let Some(base) = std::env::var_os("APPDATA") {
        PathBuf::from(base).join("RustPdf")
    } else if let Some(base) = std::env::var_os("HOME") {
        PathBuf::from(base).join(".config").join("rustpdf")
    } else {
        PathBuf::from(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_mode_tracks_detected_os_theme() {
        let theme = AppTheme {
            mode: ThemeMode::System,
            ..AppTheme::default()
        };

        assert!(theme.is_dark_for(Some(egui::Theme::Dark)));
        assert!(!theme.is_dark_for(Some(egui::Theme::Light)));
    }

    #[test]
    fn system_mode_falls_back_to_dark_when_os_theme_is_unavailable() {
        let theme = AppTheme {
            mode: ThemeMode::System,
            ..AppTheme::default()
        };

        assert!(theme.is_dark_for(None));
    }

    #[test]
    fn explicit_modes_override_detected_os_theme() {
        let dark = AppTheme {
            mode: ThemeMode::Dark,
            ..AppTheme::default()
        };
        let light = AppTheme {
            mode: ThemeMode::Light,
            ..AppTheme::default()
        };

        assert!(dark.is_dark_for(Some(egui::Theme::Light)));
        assert!(!light.is_dark_for(Some(egui::Theme::Dark)));
    }

    #[test]
    fn system_mode_uses_the_corresponding_color_palette() {
        let theme = AppTheme {
            mode: ThemeMode::System,
            ..AppTheme::default()
        };

        assert_eq!(
            theme.bg_color_for(Some(egui::Theme::Light)),
            Color32::from_rgb(theme.light_bg[0], theme.light_bg[1], theme.light_bg[2])
        );
        assert_eq!(
            theme.bg_color_for(Some(egui::Theme::Dark)),
            Color32::from_rgb(theme.dark_bg[0], theme.dark_bg[1], theme.dark_bg[2])
        );
    }
}
