//! rustpdf-ui/fonts — Universal multi-language font loader for egui.
//! Automatically discovers and registers platform system fonts for Bengali, Indic,
//! CJK (Chinese, Japanese, Korean), Arabic, Cyrillic, and world scripts to eliminate missing glyphs (□).

use egui::{FontData, FontDefinitions, FontFamily};
use std::fs;
use std::path::{Path, PathBuf};

/// Scans standard platform font paths and registers world language fallback fonts into egui.
pub fn setup_global_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    let mut loaded_indic = false;
    let mut loaded_cjk = false;
    let mut loaded_segoe = false;

    // 1. Candidate paths by OS
    let search_paths = get_platform_font_candidates();

    // 2. Load Indic / South Asian font (Bengali, Hindi, Tamil, Telugu, Gujarati, Gurmukhi, etc.)
    for path in &search_paths.indic {
        if path.exists() {
            if let Ok(bytes) = fs::read(path) {
                let mut data = FontData::from_owned(bytes);
                if path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ttc"))
                {
                    data.index = 0;
                }
                fonts
                    .font_data
                    .insert("indic_world_font".to_string(), data.into());
                loaded_indic = true;
                log::info!("Loaded Indic/Bengali system font from {:?}", path);
                break;
            }
        }
    }

    // 3. Load Modern Universal / Latin / Cyrillic / Arabic font (Segoe UI or Arial)
    for path in &search_paths.universal {
        if path.exists() {
            if let Ok(bytes) = fs::read(path) {
                let mut data = FontData::from_owned(bytes);
                if path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ttc"))
                {
                    data.index = 0;
                }
                fonts
                    .font_data
                    .insert("universal_ui_font".to_string(), data.into());
                loaded_segoe = true;
                log::info!("Loaded universal UI system font from {:?}", path);
                break;
            }
        }
    }

    // 4. Load CJK font (Chinese, Japanese, Korean)
    for path in &search_paths.cjk {
        if path.exists() {
            if let Ok(bytes) = fs::read(path) {
                let mut data = FontData::from_owned(bytes);
                if path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ttc"))
                {
                    data.index = 0;
                }
                fonts
                    .font_data
                    .insert("cjk_world_font".to_string(), data.into());
                loaded_cjk = true;
                log::info!("Loaded CJK system font from {:?}", path);
                break;
            }
        }
    }

    // 5. Inject into FontFamily fallback chains
    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        if loaded_segoe {
            proportional.push("universal_ui_font".to_string());
        }
        if loaded_indic {
            proportional.push("indic_world_font".to_string());
        }
        if loaded_cjk {
            proportional.push("cjk_world_font".to_string());
        }
    }

    if let Some(monospace) = fonts.families.get_mut(&FontFamily::Monospace) {
        if loaded_indic {
            monospace.push("indic_world_font".to_string());
        }
        if loaded_cjk {
            monospace.push("cjk_world_font".to_string());
        }
    }

    ctx.set_fonts(fonts);
}

struct PlatformFontCandidates {
    indic: Vec<PathBuf>,
    universal: Vec<PathBuf>,
    cjk: Vec<PathBuf>,
}

fn get_platform_font_candidates() -> PlatformFontCandidates {
    let mut indic = Vec::new();
    let mut universal = Vec::new();
    let mut cjk = Vec::new();

    // Windows font locations
    #[cfg(target_os = "windows")]
    {
        let win_dir = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let fonts_dir = Path::new(&win_dir).join("Fonts");

        // Indic (covers Bengali, Devanagari, Tamil, Telugu, etc.)
        indic.push(fonts_dir.join("Nirmala.ttc"));
        indic.push(fonts_dir.join("Nirmala.ttf"));
        indic.push(fonts_dir.join("NirmalaB.ttf"));
        indic.push(fonts_dir.join("vrinda.ttf"));
        indic.push(fonts_dir.join("solaimanlipi.ttf"));
        indic.push(fonts_dir.join("kalpurush.ttf"));

        // Universal (Latin, Cyrillic, Greek, Arabic, Hebrew)
        universal.push(fonts_dir.join("segoeui.ttf"));
        universal.push(fonts_dir.join("arial.ttf"));
        universal.push(fonts_dir.join("calibri.ttf"));

        // CJK
        cjk.push(fonts_dir.join("msyh.ttc"));
        cjk.push(fonts_dir.join("simsun.ttc"));
        cjk.push(fonts_dir.join("malgun.ttf"));
        cjk.push(fonts_dir.join("meiryo.ttc"));
    }

    // Unix / Linux font locations
    #[cfg(target_os = "linux")]
    {
        indic.push(PathBuf::from(
            "/usr/share/fonts/truetype/noto/NotoSansBengali-Regular.ttf",
        ));
        indic.push(PathBuf::from(
            "/usr/share/fonts/truetype/noto/NotoSansDevanagari-Regular.ttf",
        ));
        indic.push(PathBuf::from(
            "/usr/share/fonts/opentype/noto/NotoSansBengali-Regular.otf",
        ));

        universal.push(PathBuf::from(
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ));
        universal.push(PathBuf::from(
            "/usr/share/fonts/opentype/noto/NotoSans-Regular.ttf",
        ));

        cjk.push(PathBuf::from(
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ));
        cjk.push(PathBuf::from(
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        ));
    }

    // macOS font locations
    #[cfg(target_os = "macos")]
    {
        indic.push(PathBuf::from(
            "/System/Library/Fonts/Supplemental/NotoSansBengali-Regular.ttf",
        ));
        indic.push(PathBuf::from(
            "/System/Library/Fonts/Supplemental/Bangla Sangam MN.ttc",
        ));

        universal.push(PathBuf::from(
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ));
        universal.push(PathBuf::from("/System/Library/Fonts/Helvetica.ttc"));

        cjk.push(PathBuf::from("/System/Library/Fonts/PingFang.ttc"));
        cjk.push(PathBuf::from("/System/Library/Fonts/AppleSDGothicNeo.ttc"));
    }

    PlatformFontCandidates {
        indic,
        universal,
        cjk,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_font_candidates_non_empty() {
        let candidates = get_platform_font_candidates();
        assert!(!candidates.indic.is_empty());
        assert!(!candidates.universal.is_empty());
        assert!(!candidates.cjk.is_empty());
    }

    #[test]
    fn test_setup_global_fonts_headless() {
        let ctx = egui::Context::default();
        setup_global_fonts(&ctx);
    }
}
