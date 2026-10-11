//! rustpdf-ui/modals — Workbench dialogs: Combine, Split, Properties, Themes, Thumbnails, and Command Palette.

use crate::theme::{AccentPreset, AppTheme, ThemeMode};
use crate::thumbnails::{ThumbnailConfig, ThumbnailPreset};
use egui::{
    vec2, Button, Color32, Context, CornerRadius, RichText, ScrollArea, Slider, Stroke, StrokeKind,
    TextEdit, Window,
};
use rustpdf_engine::DocumentSession;
use std::path::PathBuf;

pub struct ModalsState {
    pub show_appearance: bool,
    pub show_thumbnails: bool,
    pub show_properties: bool,
    pub show_combine: bool,
    pub show_split: bool,
    pub show_command_palette: bool,
    pub show_about: bool,
    pub show_protect: bool,
    pub show_forms_manager: bool,
    pub show_layers_manager: bool,
    pub show_annotations_manager: bool,

    // Combine dialog state
    pub combine_inputs: Vec<PathBuf>,

    // Split dialog state
    pub split_every_n: usize,
    pub split_out_dir: PathBuf,

    // Command palette state
    pub command_query: String,

    // Protection dialog state
    pub protect_user_pass: String,
    pub protect_owner_pass: String,
    pub protect_allow_print: bool,
    pub protect_allow_copy: bool,
    pub protect_allow_modify: bool,
    pub protect_allow_annotate: bool,

    // Annotation dialog state
    pub new_annot_text: String,
    pub new_annot_author: String,

    // Fill & Sign: Signature & Initials dialog state
    pub show_signature_dialog: bool,
    pub signature_text: String,
    pub is_initials_mode: bool,

    // Edit Content & Images Dialog State
    pub show_edit_content_dialog: bool,
    pub edit_content_tab: EditContentTab,
    pub edit_text_input: String,
    pub edit_text_size: f32,
    pub edit_text_x: f32,
    pub edit_text_y: f32,
    pub edit_find_query: String,
    pub edit_replace_query: String,
    pub edit_image_path: Option<PathBuf>,
    pub edit_image_x: f32,
    pub edit_image_y: f32,
    pub edit_image_w: f32,
    pub edit_image_h: f32,
    pub edit_watermark_text: String,
    pub edit_watermark_size: f32,
    pub edit_watermark_rotation: f32,
    pub edit_header_text: String,
    pub edit_footer_text: String,
    pub edit_bates_prefix: String,
    pub edit_bates_start: u32,
    pub edit_link_url: String,

    // Target page selection for modals that act on pages (1-based for user display)
    pub target_page: usize,
    pub apply_to_all_pages: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditContentTab {
    Text,
    Image,
    Watermark,
    HeaderFooter,
    Background,
    Bates,
    Link,
}

impl Default for ModalsState {
    fn default() -> Self {
        Self {
            show_appearance: false,
            show_thumbnails: false,
            show_properties: false,
            show_combine: false,
            show_split: false,
            show_command_palette: false,
            show_about: false,
            show_protect: false,
            show_forms_manager: false,
            show_layers_manager: false,
            show_annotations_manager: false,
            show_signature_dialog: false,
            signature_text: String::from("John Doe"),
            is_initials_mode: false,
            show_edit_content_dialog: false,
            edit_content_tab: EditContentTab::Text,
            edit_text_input: String::from("Hello, RustPDF!"),
            edit_text_size: 14.0,
            edit_text_x: 72.0,
            edit_text_y: 700.0,
            edit_find_query: String::new(),
            edit_replace_query: String::new(),
            edit_image_path: None,
            edit_image_x: 72.0,
            edit_image_y: 500.0,
            edit_image_w: 200.0,
            edit_image_h: 150.0,
            edit_watermark_text: String::from("CONFIDENTIAL"),
            edit_watermark_size: 48.0,
            edit_watermark_rotation: 45.0,
            edit_header_text: String::from("RustPDF Document"),
            edit_footer_text: String::from("Page {n} of {total}"),
            edit_bates_prefix: String::from("DOC-"),
            edit_bates_start: 1,
            edit_link_url: String::from("https://github.com/Sami12901/RustPDF"),
            target_page: 1,
            apply_to_all_pages: false,
            combine_inputs: Vec::new(),
            split_every_n: 1,
            split_out_dir: PathBuf::from("."),
            command_query: String::new(),
            protect_user_pass: String::new(),
            protect_owner_pass: String::new(),
            protect_allow_print: true,
            protect_allow_copy: true,
            protect_allow_modify: true,
            protect_allow_annotate: true,
            new_annot_text: String::new(),
            new_annot_author: String::from("User"),
        }
    }
}

pub enum ModalAction {
    Combine(Vec<PathBuf>),
    Split(usize, PathBuf),
    ExecuteCommand(String),
    DocumentModified,
}

pub struct ModalsComponent;

impl ModalsComponent {
    pub fn render(
        ctx: &Context,
        state: &mut ModalsState,
        theme: &mut AppTheme,
        thumbnails: &mut ThumbnailConfig,
        mut session: Option<&mut DocumentSession>,
    ) -> Option<ModalAction> {
        let mut action = None;

        // 1. Appearance / Theme Customizer Modal
        if state.show_appearance {
            Window::new("🎨 Appearance Settings")
                .open(&mut state.show_appearance)
                .resizable(true)
                .default_width(420.0)
                .show(ctx, |ui| {
                    ui.heading("Theme Mode");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut theme.mode, ThemeMode::Dark, "🌙 Dark");
                        ui.radio_value(&mut theme.mode, ThemeMode::Light, "☀️ Light");
                        ui.radio_value(&mut theme.mode, ThemeMode::System, "🖥️ System");
                    });

                    ui.add_space(14.0);
                    ui.heading("Accent Color");
                    ui.horizontal_wrapped(|ui| {
                        let presets = [
                            AccentPreset::RustOrange,
                            AccentPreset::Blue,
                            AccentPreset::Green,
                            AccentPreset::Violet,
                            AccentPreset::Teal,
                            AccentPreset::Custom,
                        ];

                        for p in presets {
                            if ui
                                .selectable_label(theme.accent_preset == p, p.display_name())
                                .clicked()
                            {
                                theme.accent_preset = p;
                            }
                        }
                    });

                    if theme.accent_preset == AccentPreset::Custom {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label("Pick Color:");
                            ui.color_edit_button_srgb(&mut theme.custom_accent);
                        });
                    }

                    ui.add_space(16.0);
                    ui.heading("Live Token Preview");
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Accent Sample")
                                .color(theme.active_accent())
                                .strong(),
                        );
                        ui.label(RichText::new("Primary Text").color(theme.text_primary()));
                        ui.label(RichText::new("Secondary Text").color(theme.text_secondary()));
                    });

                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save Theme").clicked() {
                            let _ = theme.save_persisted();
                            theme.apply_to_egui(ctx);
                        }

                        if ui.button("↺ Reset to Default").clicked() {
                            *theme = AppTheme::default();
                            theme.apply_to_egui(ctx);
                        }

                        if ui.button("📤 Export JSON").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON Theme", &["json"])
                                .save_file()
                            {
                                if let Ok(data) = serde_json::to_string_pretty(theme) {
                                    let _ = std::fs::write(path, data);
                                }
                            }
                        }

                        if ui.button("📥 Import JSON").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON Theme", &["json"])
                                .pick_file()
                            {
                                if let Ok(data) = std::fs::read_to_string(path) {
                                    if let Ok(imported) = serde_json::from_str(&data) {
                                        *theme = imported;
                                        theme.apply_to_egui(ctx);
                                    }
                                }
                            }
                        }
                    });
                });
        }

        // 2. Thumbnail Settings Modal
        if state.show_thumbnails {
            Window::new("🖼️ Thumbnail Settings")
                .open(&mut state.show_thumbnails)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.heading("Layout Presets");
                    ui.horizontal(|ui| {
                        if ui.button("Compact").clicked() {
                            thumbnails.apply_preset(ThumbnailPreset::Compact);
                        }
                        if ui.button("Balanced").clicked() {
                            thumbnails.apply_preset(ThumbnailPreset::Balanced);
                        }
                        if ui.button("Large Previews").clicked() {
                            thumbnails.apply_preset(ThumbnailPreset::Large);
                        }
                    });

                    ui.add_space(14.0);
                    ui.heading("Custom Dimensions");

                    ui.add(
                        Slider::new(&mut thumbnails.size, 80.0..=320.0).text("Thumbnail Size (pt)"),
                    );
                    ui.add(Slider::new(&mut thumbnails.density, 0.5..=2.0).text("Grid Density"));
                    ui.add(
                        Slider::new(&mut thumbnails.sidebar_width, 120.0..=360.0)
                            .text("Sidebar Width (pt)"),
                    );
                    ui.add(
                        Slider::new(&mut thumbnails.corner_radius, 0.0..=16.0)
                            .text("Corner Radius (px)"),
                    );

                    ui.add_space(10.0);
                    ui.checkbox(&mut thumbnails.show_shadow, "Card Drop Shadow");
                    ui.checkbox(&mut thumbnails.show_page_labels, "Page Number Labels");
                    ui.checkbox(&mut thumbnails.show_dimensions, "Page Dimensions Label");

                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save Settings").clicked() {
                            let _ = thumbnails.save_persisted();
                        }
                        if ui.button("↺ Reset Defaults").clicked() {
                            *thumbnails = ThumbnailConfig::default();
                        }
                    });
                });
        }

        // 3. Document Properties Modal
        if state.show_properties {
            if let Some(ref mut s) = session {
                Window::new("ℹ️ Document Properties")
                    .open(&mut state.show_properties)
                    .resizable(true)
                    .default_width(460.0)
                    .show(ctx, |ui| {
                        ui.heading("Document Description");

                        let mut title = s.model.metadata.title.clone().unwrap_or_default();
                        let mut author = s.model.metadata.author.clone().unwrap_or_default();
                        let mut subject = s.model.metadata.subject.clone().unwrap_or_default();
                        let mut keywords = s.model.metadata.keywords.clone().unwrap_or_default();

                        ui.horizontal(|ui| {
                            ui.label("Title:   ");
                            ui.text_edit_singleline(&mut title);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Author:  ");
                            ui.text_edit_singleline(&mut author);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Subject: ");
                            ui.text_edit_singleline(&mut subject);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Keywords:");
                            ui.text_edit_singleline(&mut keywords);
                        });

                        ui.add_space(14.0);
                        ui.heading("Technical Details");
                        ui.label(format!(
                            "PDF Version:   PDF-{}",
                            s.model.metadata.pdf_version
                        ));
                        ui.label(format!("Page Count:    {}", s.page_count()));
                        ui.label(format!("Encrypted:     {}", s.model.metadata.encrypted));
                        ui.label(format!(
                            "File Size:     {} bytes",
                            s.model.metadata.file_size_bytes
                        ));

                        ui.add_space(16.0);
                        if ui.button("💾 Apply & Save Metadata").clicked() {
                            let mut updated = s.model.metadata.clone();
                            updated.title = if title.is_empty() { None } else { Some(title) };
                            updated.author = if author.is_empty() {
                                None
                            } else {
                                Some(author)
                            };
                            updated.subject = if subject.is_empty() {
                                None
                            } else {
                                Some(subject)
                            };
                            updated.keywords = if keywords.is_empty() {
                                None
                            } else {
                                Some(keywords)
                            };
                            let _ = s.update_metadata(updated);
                        }
                    });
            }
        }

        // 4. Combine Files Dialog
        if state.show_combine {
            let mut close_combine = false;
            Window::new("📑 Combine PDF Files")
                .open(&mut state.show_combine)
                .resizable(true)
                .default_width(450.0)
                .show(ctx, |ui| {
                    ui.label("Add PDF files to merge into a single document:");

                    if ui.button("➕ Add File(s)...").clicked() {
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter("PDF Documents", &["pdf"])
                            .pick_files()
                        {
                            state.combine_inputs.extend(files);
                        }
                    }

                    ui.add_space(8.0);
                    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        let mut to_remove = None;
                        for (i, p) in state.combine_inputs.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "{}. {}",
                                    i + 1,
                                    p.file_name().unwrap_or_default().to_string_lossy()
                                ));
                                if ui.button("✖").clicked() {
                                    to_remove = Some(i);
                                }
                            });
                        }
                        if let Some(idx) = to_remove {
                            state.combine_inputs.remove(idx);
                        }
                    });

                    ui.add_space(14.0);
                    if ui
                        .add_enabled(!state.combine_inputs.is_empty(), Button::new("Combine Now"))
                        .clicked()
                    {
                        action = Some(ModalAction::Combine(state.combine_inputs.clone()));
                        close_combine = true;
                    }
                });
            if close_combine {
                state.show_combine = false;
            }
        }

        // 5. Split Document Dialog
        if state.show_split {
            let mut close_split = false;
            Window::new("✂️ Split Document")
                .open(&mut state.show_split)
                .resizable(false)
                .default_width(380.0)
                .show(ctx, |ui| {
                    ui.label("Split current document into multiple files:");

                    ui.add(Slider::new(&mut state.split_every_n, 1..=50).text("Pages per file"));

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("Output: {}", state.split_out_dir.display()));
                        if ui.button("Browse...").clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                state.split_out_dir = folder;
                            }
                        }
                    });

                    ui.add_space(16.0);
                    if ui.button("Split Now").clicked() {
                        action = Some(ModalAction::Split(
                            state.split_every_n,
                            state.split_out_dir.clone(),
                        ));
                        close_split = true;
                    }
                });
            if close_split {
                state.show_split = false;
            }
        }

        // 6. Command Palette (<kbd>Ctrl+K</kbd>)
        if state.show_command_palette {
            let mut close_palette = false;
            Window::new("⌨️ Command Palette")
                .open(&mut state.show_command_palette)
                .resizable(false)
                .default_width(450.0)
                .anchor(egui::Align2::CENTER_TOP, vec2(0.0, 100.0))
                .show(ctx, |ui| {
                    ui.add(
                        TextEdit::singleline(&mut state.command_query)
                            .hint_text("Type a command (Open, Save, Organize, Appearance...)...")
                            .desired_width(ui.available_width()),
                    );

                    ui.add_space(8.0);
                    let commands = [
                        ("Open PDF", "Open a document from disk"),
                        ("Save Document", "Save changes to current document"),
                        ("Save As...", "Save document with new filename"),
                        ("Organize Pages", "Switch to full page organization grid"),
                        ("Find in Document", "Search text within current document"),
                        ("Combine Files", "Merge multiple PDFs into one"),
                        ("Split Document", "Split document by page count"),
                        (
                            "Appearance Settings",
                            "Customize dark/light theme and accent colors",
                        ),
                        (
                            "Thumbnail Settings",
                            "Adjust thumbnail sizing, grid, and borders",
                        ),
                        ("Document Properties", "Inspect and edit PDF metadata"),
                    ];

                    let query = state.command_query.to_lowercase();
                    for (cmd, desc) in commands {
                        if query.is_empty()
                            || cmd.to_lowercase().contains(&query)
                            || desc.to_lowercase().contains(&query)
                        {
                            if ui
                                .selectable_label(false, format!("▶ {cmd} — {desc}"))
                                .clicked()
                            {
                                action = Some(ModalAction::ExecuteCommand(cmd.to_string()));
                                close_palette = true;
                            }
                        }
                    }
                });
            if close_palette {
                state.show_command_palette = false;
                state.command_query.clear();
            }
        }

        // 7. About RustPdf Modal (Features the In-App Brand Logo)
        if state.show_about {
            let mut is_open = true;
            let mut close_dialog = false;
            Window::new("About RustPdf")
                .open(&mut is_open)
                .resizable(false)
                .default_width(380.0)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        let (logo_rect, _) = ui.allocate_exact_size(vec2(72.0, 80.0), egui::Sense::hover());
                        crate::icons::Icons::draw_logo(ui.painter(), logo_rect, theme.active_accent());
                        ui.add_space(10.0);
                        ui.heading(
                            RichText::new("RustPdf")
                                .size(26.0)
                                .strong()
                                .color(theme.active_accent()),
                        );
                        ui.label(
                            RichText::new("Version 0.1.0 • Clean-Room PDF Workbench")
                                .size(12.0)
                                .color(Color32::from_gray(160)),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("100% Offline • Zero Telemetry • Pure Rust")
                                .size(11.0)
                                .color(Color32::from_rgb(46, 180, 120)),
                        );
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label("RustPdf is a high-performance, memory-safe PDF editing workbench inspired by the capabilities of PdfCraft.");
                    ui.add_space(4.0);
                    ui.label(RichText::new("Dual-licensed under MIT OR Apache-2.0.").size(11.0).color(Color32::from_gray(140)));
                    ui.label(RichText::new("Designed & built with pure Rust, egui, lopdf & hayro.").size(11.0).color(Color32::from_gray(140)));

                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        if ui.button("Close").clicked() {
                            close_dialog = true;
                        }
                    });
                });
            state.show_about = is_open && !close_dialog;
        }

        // 8. Protect & Security Dialog
        if state.show_protect {
            if let Some(ref mut s) = session {
                let mut is_open = true;
                let mut close_dialog = false;
                Window::new("Protect Document with Password")
                    .open(&mut is_open)
                    .resizable(false)
                    .default_width(420.0)
                    .show(ctx, |ui| {
                        ui.label("Encrypt document and set access permissions:");
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label("User Password: ");
                            ui.add(
                                TextEdit::singleline(&mut state.protect_user_pass).password(true),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Owner Password:");
                            ui.add(
                                TextEdit::singleline(&mut state.protect_owner_pass).password(true),
                            );
                        });

                        ui.add_space(10.0);
                        ui.heading("Permissions");
                        ui.checkbox(&mut state.protect_allow_print, "Allow Printing");
                        ui.checkbox(&mut state.protect_allow_copy, "Allow Copying Content");
                        ui.checkbox(&mut state.protect_allow_modify, "Allow Modifying Pages");
                        ui.checkbox(
                            &mut state.protect_allow_annotate,
                            "Allow Annotating & Forms",
                        );

                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            if ui.button("Encrypt & Protect").clicked() {
                                let perms = rustpdf_cos::security::DocumentPermissions {
                                    allow_print: state.protect_allow_print,
                                    allow_copy: state.protect_allow_copy,
                                    allow_modify: state.protect_allow_modify,
                                    allow_annotate: state.protect_allow_annotate,
                                    ..Default::default()
                                };
                                let _ = s.protect_document(
                                    &state.protect_user_pass,
                                    &state.protect_owner_pass,
                                    perms,
                                );
                                close_dialog = true;
                            }

                            if ui.button("Remove Protection").clicked() {
                                let _ = s.remove_protection();
                                close_dialog = true;
                            }
                        });
                    });
                state.show_protect = is_open && !close_dialog;
            }
        }

        // 9. Interactive Forms Manager
        if state.show_forms_manager {
            if let Some(ref mut s) = session {
                Window::new("Interactive Form Fields")
                    .open(&mut state.show_forms_manager)
                    .resizable(true)
                    .default_width(480.0)
                    .show(ctx, |ui| {
                        let fields = s.get_form_fields();
                        if fields.is_empty() {
                            ui.label("No interactive form fields found in this document.");
                        } else {
                            ui.label(format!("Found {} form field(s):", fields.len()));
                            ui.add_space(6.0);
                            ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                                for f in fields {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&f.name).strong());
                                        ui.label(format!("({:?})", f.kind));
                                        let val = f.value.unwrap_or_default();
                                        ui.label(format!("= \"{val}\""));
                                    });
                                }
                            });
                        }

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("Reset All Fields").clicked() {
                                let _ = s.reset_form_fields();
                            }
                            if ui.button("Export Form Data (JSON)").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("JSON", &["json"])
                                    .save_file()
                                {
                                    if let Ok(json) =
                                        rustpdf_forms::export_form_data_json(s.cos_doc.inner())
                                    {
                                        let _ = std::fs::write(path, json);
                                    }
                                }
                            }
                        });
                    });
            }
        }

        // 10. Layers & Attachments Manager
        if state.show_layers_manager {
            if let Some(ref mut s) = session {
                Window::new("Layers & Attachments")
                    .open(&mut state.show_layers_manager)
                    .resizable(true)
                    .default_width(460.0)
                    .show(ctx, |ui| {
                        ui.heading("Optional Content Layers");
                        let layers = s.model.layers.clone();
                        if layers.is_empty() {
                            ui.label(
                                RichText::new("No optional content layers in document.")
                                    .color(Color32::from_gray(140)),
                            );
                        } else {
                            for layer in &layers {
                                ui.label(format!("• {} (Visible: {})", layer.name, layer.visible));
                            }
                        }

                        ui.add_space(14.0);
                        ui.heading("Embedded Attachments");
                        let attachments = s.model.attachments.clone();
                        if attachments.is_empty() {
                            ui.label(
                                RichText::new("No embedded attachments in document.")
                                    .color(Color32::from_gray(140)),
                            );
                        } else {
                            for att in &attachments {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "Attachment: {} ({} bytes)",
                                        att.filename, att.size_bytes
                                    ));
                                });
                            }
                        }

                        ui.add_space(12.0);
                        if ui.button("Attach File...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                if let Ok(bytes) = std::fs::read(&path) {
                                    let filename = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                    let _ = s.add_attachment(&filename, &bytes);
                                }
                            }
                        }
                    });
            }
        }

        // 11. Comments & Annotations Manager
        if state.show_annotations_manager {
            if let Some(ref mut s) = session {
                let total_pages = s.page_count().max(1);
                let target_idx = (state.target_page.clamp(1, total_pages)) - 1;

                Window::new("Comments & Annotations")
                    .open(&mut state.show_annotations_manager)
                    .resizable(true)
                    .default_width(440.0)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Page:");
                            ui.add(egui::DragValue::new(&mut state.target_page).range(1..=total_pages));
                            ui.label(format!("of {}", total_pages));
                        });
                        ui.separator();

                        ui.heading("Add Sticky Note / Comment");
                        ui.horizontal(|ui| {
                            ui.label("Author: ");
                            ui.text_edit_singleline(&mut state.new_annot_author);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Comment:");
                            ui.text_edit_singleline(&mut state.new_annot_text);
                        });

                        ui.add_space(6.0);
                        if ui.button(format!("➕ Add Note to Page {}", target_idx + 1)).clicked() {
                            if !state.new_annot_text.is_empty() {
                                let annot = rustpdf_annot::Annotation {
                                    id: None,
                                    page_index: target_idx,
                                    annot_type: rustpdf_annot::AnnotType::Text,
                                    rect: rustpdf_geom::Rect::new(100.0, 700.0, 140.0, 740.0),
                                    color: [1.0, 0.85, 0.0],
                                    contents: Some(state.new_annot_text.clone()),
                                    author: Some(state.new_annot_author.clone()),
                                    date: Some(
                                        chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                    ),
                                    ink_paths: None,
                                };
                                let _ = s.add_annotation(&annot);
                                state.new_annot_text.clear();
                                action = Some(ModalAction::DocumentModified);
                            }
                        }

                        ui.add_space(12.0);
                        ui.heading(format!("Page {} Annotations", target_idx + 1));
                        let annots = s.get_annotations(target_idx);
                        if annots.is_empty() {
                            ui.label(
                                RichText::new("No annotations on this page.")
                                    .color(Color32::from_gray(140)),
                            );
                        } else {
                            let mut to_delete = None;
                            ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                for a in &annots {
                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "[{:?}] {}",
                                            a.annot_type,
                                            a.contents.as_deref().unwrap_or("No content")
                                        ));
                                        if let Some(ref_id) = a.id {
                                             if ui.button("🗑").clicked() {
                                                 to_delete = Some(ref_id);
                                             }
                                        }
                                    });
                                }
                            });
                            if let Some(id) = to_delete {
                                let _ = s.delete_annotation(id);
                                action = Some(ModalAction::DocumentModified);
                            }
                        }
                    });
            }
        }

        // 12. Fill & Sign: Signature & Initials Dialog
        if state.show_signature_dialog {
            let mut close_signature = false;
            let title = if state.is_initials_mode {
                "✍ Add Initials"
            } else {
                "✍ Add Signature"
            };
            Window::new(title)
                .open(&mut state.show_signature_dialog)
                .resizable(true)
                .default_width(380.0)
                .show(ctx, |ui| {
                    if let Some(ref s) = session {
                        let total_pages = s.page_count().max(1);
                        ui.horizontal(|ui| {
                            ui.label("Stamp on Page:");
                            ui.add(egui::DragValue::new(&mut state.target_page).range(1..=total_pages));
                            ui.label(format!("of {}", total_pages));
                        });
                        ui.separator();
                    }

                    ui.add_space(4.0);
                    let prompt = if state.is_initials_mode {
                        "Enter your initials:"
                    } else {
                        "Enter your name / signature text:"
                    };
                    ui.label(RichText::new(prompt).strong());
                    ui.add_space(4.0);
                    ui.text_edit_singleline(&mut state.signature_text);
                    ui.add_space(10.0);

                    // Cursive handwritten-style preview
                    ui.label(
                        RichText::new("Preview:")
                            .size(12.0)
                            .color(Color32::from_gray(140)),
                    );
                    let (preview_rect, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 64.0),
                        egui::Sense::hover(),
                    );
                    let is_dark = ui.visuals().dark_mode;
                    let bg = if is_dark {
                        Color32::from_rgb(32, 35, 45)
                    } else {
                        Color32::from_rgb(245, 246, 250)
                    };
                    ui.painter().rect(
                        preview_rect,
                        CornerRadius::same(6),
                        bg,
                        Stroke::new(1.0, Color32::from_gray(160)),
                        StrokeKind::Inside,
                    );
                    let sig_val = if state.signature_text.trim().is_empty() {
                        if state.is_initials_mode {
                            "J.D."
                        } else {
                            "John Doe"
                        }
                    } else {
                        &state.signature_text
                    };
                    let sig_color = if is_dark {
                        Color32::from_rgb(147, 197, 253)
                    } else {
                        Color32::from_rgb(29, 78, 216)
                    };
                    let mut sig_child =
                        ui.new_child(egui::UiBuilder::new().max_rect(preview_rect).layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        ));
                    sig_child.label(RichText::new(sig_val).size(22.0).italics().color(sig_color));

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if let Some(ref mut s) = session {
                            let total_pages = s.page_count().max(1);
                            let target_idx = (state.target_page.clamp(1, total_pages)) - 1;
                            let btn_label = if state.is_initials_mode {
                                format!("🖋 Stamp Initials to Page {}", target_idx + 1)
                            } else {
                                format!("🖋 Stamp Signature to Page {}", target_idx + 1)
                            };
                            if ui.button(RichText::new(btn_label).strong()).clicked() {
                                let content = if state.is_initials_mode {
                                    format!("Initials: {}", sig_val)
                                } else {
                                    format!("Signature: {}", sig_val)
                                };
                                let annot = rustpdf_annot::Annotation {
                                    id: None,
                                    page_index: target_idx,
                                    annot_type: rustpdf_annot::AnnotType::Text,
                                    rect: rustpdf_geom::Rect::new(100.0, 650.0, 300.0, 690.0),
                                    color: [0.12, 0.45, 0.90],
                                    contents: Some(content),
                                    author: Some(sig_val.to_string()),
                                    date: Some(
                                        chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                    ),
                                    ink_paths: None,
                                };
                                let _ = s.add_annotation(&annot);
                                close_signature = true;
                                action = Some(ModalAction::DocumentModified);
                            }
                        } else {
                            ui.label(
                                RichText::new("Open a PDF to place signature")
                                    .color(Color32::from_gray(140)),
                            );
                        }
                        if ui.button("Close").clicked() {
                            close_signature = true;
                        }
                    });
                });
            if close_signature {
                state.show_signature_dialog = false;
            }
        }

        // 13. Edit Text & Images Dialog
        if state.show_edit_content_dialog {
            let mut close_edit = false;
            Window::new("✏️ Edit Text & Images")
                .open(&mut state.show_edit_content_dialog)
                .resizable(true)
                .default_width(520.0)
                .show(ctx, |ui| {
                    let (total_pages, target_idx) = if let Some(ref s) = session {
                        let tot = s.page_count().max(1);
                        let idx = (state.target_page.clamp(1, tot)) - 1;
                        (tot, idx)
                    } else {
                        (1, 0)
                    };

                    ui.horizontal(|ui| {
                        ui.label("Target Page:");
                        ui.add(egui::DragValue::new(&mut state.target_page).range(1..=total_pages));
                        ui.label(format!("of {}", total_pages));
                    });
                    ui.separator();

                    // Navigation Tabs
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(
                            &mut state.edit_content_tab,
                            EditContentTab::Text,
                            "✏ Text",
                        );
                        ui.selectable_value(
                            &mut state.edit_content_tab,
                            EditContentTab::Image,
                            "🖼 Image",
                        );
                        ui.selectable_value(
                            &mut state.edit_content_tab,
                            EditContentTab::Watermark,
                            "🌊 Watermark",
                        );
                        ui.selectable_value(
                            &mut state.edit_content_tab,
                            EditContentTab::HeaderFooter,
                            "📄 Header & Footer",
                        );
                        ui.selectable_value(
                            &mut state.edit_content_tab,
                            EditContentTab::Background,
                            "🎨 Background",
                        );
                        ui.selectable_value(
                            &mut state.edit_content_tab,
                            EditContentTab::Bates,
                            "🔢 Bates Number",
                        );
                        ui.selectable_value(
                            &mut state.edit_content_tab,
                            EditContentTab::Link,
                            "🔗 Link",
                        );
                    });
                    ui.separator();
                    ui.add_space(6.0);

                    match state.edit_content_tab {
                        EditContentTab::Text => {
                            ui.heading(format!("Add New Text Box (Page {})", target_idx + 1));
                            ui.horizontal(|ui| {
                                ui.label("Text:     ");
                                ui.text_edit_singleline(&mut state.edit_text_input);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Font Size:");
                                ui.add(
                                    Slider::new(&mut state.edit_text_size, 8.0..=72.0)
                                        .suffix(" pt"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Position X:");
                                ui.add(
                                    Slider::new(&mut state.edit_text_x, 0.0..=600.0).suffix(" pt"),
                                );
                                ui.label("Y:");
                                ui.add(
                                    Slider::new(&mut state.edit_text_y, 0.0..=800.0).suffix(" pt"),
                                );
                            });

                            ui.add_space(8.0);
                            if let Some(ref mut s) = session {
                                if ui
                                    .button(RichText::new(format!("➕ Apply Text to Page {}", target_idx + 1)).strong())
                                    .clicked()
                                {
                                    let _ = s.add_text_to_page(
                                        target_idx,
                                        &state.edit_text_input,
                                        state.edit_text_x,
                                        state.edit_text_y,
                                        rustpdf_fonts::StandardFont::Helvetica,
                                        state.edit_text_size,
                                        [0.1, 0.1, 0.1],
                                    );
                                    close_edit = true;
                                    action = Some(ModalAction::DocumentModified);
                                }
                            }

                            ui.separator();
                            ui.heading(format!("Find & Replace Text in Page {}", target_idx + 1));
                            ui.horizontal(|ui| {
                                ui.label("Find:   ");
                                ui.text_edit_singleline(&mut state.edit_find_query);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Replace:");
                                ui.text_edit_singleline(&mut state.edit_replace_query);
                            });

                            if let Some(ref mut s) = session {
                                if ui.button(format!("🔄 Replace Text in Page {}", target_idx + 1)).clicked() {
                                    let _ = s.search_and_replace_text(
                                        target_idx,
                                        &state.edit_find_query,
                                        &state.edit_replace_query,
                                    );
                                    close_edit = true;
                                    action = Some(ModalAction::DocumentModified);
                                }
                            }
                        }

                        EditContentTab::Image => {
                            ui.heading(format!("Embed Image on Page {}", target_idx + 1));
                            if ui.button("📁 Choose Image File...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Image", &["png", "jpg", "jpeg"])
                                    .pick_file()
                                {
                                    state.edit_image_path = Some(path);
                                }
                            }
                            if let Some(ref p) = state.edit_image_path {
                                ui.label(format!(
                                    "Selected: {}",
                                    p.file_name().unwrap_or_default().to_string_lossy()
                                ));
                            }

                            ui.horizontal(|ui| {
                                ui.label("Width: ");
                                ui.add(
                                    Slider::new(&mut state.edit_image_w, 20.0..=500.0)
                                        .suffix(" pt"),
                                );
                                ui.label("Height:");
                                ui.add(
                                    Slider::new(&mut state.edit_image_h, 20.0..=500.0)
                                        .suffix(" pt"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("X:     ");
                                ui.add(
                                    Slider::new(&mut state.edit_image_x, 0.0..=600.0).suffix(" pt"),
                                );
                                ui.label("Y:     ");
                                ui.add(
                                    Slider::new(&mut state.edit_image_y, 0.0..=800.0).suffix(" pt"),
                                );
                            });

                            ui.add_space(8.0);
                            if let Some(ref mut s) = session {
                                let has_img = state.edit_image_path.is_some();
                                if ui
                                    .add_enabled(has_img, Button::new(format!("🖼 Insert Image onto Page {}", target_idx + 1)))
                                    .clicked()
                                {
                                    if let Some(ref img_p) = state.edit_image_path {
                                        if let Ok(bytes) = std::fs::read(img_p) {
                                            let _ = s.insert_image_to_page(
                                                target_idx,
                                                &bytes,
                                                state.edit_image_x,
                                                state.edit_image_y,
                                                state.edit_image_w,
                                                state.edit_image_h,
                                            );
                                            close_edit = true;
                                            action = Some(ModalAction::DocumentModified);
                                        }
                                    }
                                }
                            }
                        }

                        EditContentTab::Watermark => {
                            ui.heading("Add Document Watermark");
                            ui.checkbox(&mut state.apply_to_all_pages, "Apply to all pages in document");
                            ui.horizontal(|ui| {
                                ui.label("Watermark Text:");
                                ui.text_edit_singleline(&mut state.edit_watermark_text);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Font Size:     ");
                                ui.add(
                                    Slider::new(&mut state.edit_watermark_size, 20.0..=120.0)
                                        .suffix(" pt"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Rotation Angle:");
                                ui.add(
                                    Slider::new(&mut state.edit_watermark_rotation, -90.0..=90.0)
                                        .suffix("°"),
                                );
                            });

                            ui.add_space(8.0);
                            if let Some(ref mut s) = session {
                                let btn_label = if state.apply_to_all_pages {
                                    "🌊 Apply Watermark to All Pages".to_string()
                                } else {
                                    format!("🌊 Apply Watermark to Page {}", target_idx + 1)
                                };
                                if ui.button(btn_label).clicked() {
                                    let pages: Vec<usize> = if state.apply_to_all_pages {
                                        (0..s.page_count()).collect()
                                    } else {
                                        vec![target_idx]
                                    };
                                    let _ = s.add_watermark(
                                        &pages,
                                        &state.edit_watermark_text,
                                        state.edit_watermark_size,
                                        [0.75, 0.75, 0.85],
                                        state.edit_watermark_rotation,
                                    );
                                    close_edit = true;
                                    action = Some(ModalAction::DocumentModified);
                                }
                            }
                        }

                        EditContentTab::HeaderFooter => {
                            ui.heading("Header & Footer");
                            ui.checkbox(&mut state.apply_to_all_pages, "Apply to all pages in document");
                            ui.label(
                                "Use {n} for current page number and {total} for total page count.",
                            );
                            ui.horizontal(|ui| {
                                ui.label("Header Text:");
                                ui.text_edit_singleline(&mut state.edit_header_text);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Footer Text:");
                                ui.text_edit_singleline(&mut state.edit_footer_text);
                            });

                            ui.add_space(8.0);
                            if let Some(ref mut s) = session {
                                let btn_label = if state.apply_to_all_pages {
                                    "📄 Apply Header & Footer to All Pages".to_string()
                                } else {
                                    format!("📄 Apply Header & Footer to Page {}", target_idx + 1)
                                };
                                if ui.button(btn_label).clicked() {
                                    let pages: Vec<usize> = if state.apply_to_all_pages {
                                        (0..s.page_count()).collect()
                                    } else {
                                        vec![target_idx]
                                    };
                                    let _ = s.add_header_footer(
                                        &pages,
                                        Some(&state.edit_header_text),
                                        Some(&state.edit_footer_text),
                                        10.0,
                                        [0.3, 0.3, 0.3],
                                    );
                                    close_edit = true;
                                    action = Some(ModalAction::DocumentModified);
                                }
                            }
                        }

                        EditContentTab::Background => {
                            ui.heading("Page Background Color");
                            ui.checkbox(&mut state.apply_to_all_pages, "Apply to all pages in document");
                            ui.label(
                                "Adds a solid colored backdrop layer behind existing contents:",
                            );
                            ui.add_space(6.0);
                            if let Some(ref mut s) = session {
                                let pages: Vec<usize> = if state.apply_to_all_pages {
                                    (0..s.page_count()).collect()
                                } else {
                                    vec![target_idx]
                                };
                                ui.horizontal(|ui| {
                                    if ui.button("🍦 Warm Cream (#FFFDF0)").clicked() {
                                        let _ = s.add_background(&pages, [1.0, 0.99, 0.94]);
                                        close_edit = true;
                                        action = Some(ModalAction::DocumentModified);
                                    }
                                    if ui.button("🌫 Soft Gray (#F4F4F6)").clicked() {
                                        let _ = s.add_background(&pages, [0.95, 0.95, 0.96]);
                                        close_edit = true;
                                        action = Some(ModalAction::DocumentModified);
                                    }
                                    if ui.button("❄ Pure White (#FFFFFF)").clicked() {
                                        let _ = s.add_background(&pages, [1.0, 1.0, 1.0]);
                                        close_edit = true;
                                        action = Some(ModalAction::DocumentModified);
                                    }
                                });
                            }
                        }

                        EditContentTab::Bates => {
                            ui.heading("Bates Numbering");
                            ui.checkbox(&mut state.apply_to_all_pages, "Apply to all pages in document");
                            ui.horizontal(|ui| {
                                ui.label("Prefix:       ");
                                ui.text_edit_singleline(&mut state.edit_bates_prefix);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Start Number: ");
                                ui.add(egui::DragValue::new(&mut state.edit_bates_start));
                            });

                            ui.add_space(8.0);
                            if let Some(ref mut s) = session {
                                let btn_label = if state.apply_to_all_pages {
                                    "🔢 Apply Bates Numbering to All Pages".to_string()
                                } else {
                                    format!("🔢 Apply Bates Numbering to Page {}", target_idx + 1)
                                };
                                if ui.button(btn_label).clicked() {
                                    let pages: Vec<usize> = if state.apply_to_all_pages {
                                        (0..s.page_count()).collect()
                                    } else {
                                        vec![target_idx]
                                    };
                                    let _ = s.add_bates_numbering(
                                        &pages,
                                        &state.edit_bates_prefix,
                                        state.edit_bates_start,
                                        6,
                                    );
                                    close_edit = true;
                                    action = Some(ModalAction::DocumentModified);
                                }
                            }
                        }

                        EditContentTab::Link => {
                            ui.heading(format!("Hyperlink Annotation (Page {})", target_idx + 1));
                            ui.horizontal(|ui| {
                                ui.label("URL: ");
                                ui.text_edit_singleline(&mut state.edit_link_url);
                            });

                            ui.add_space(8.0);
                            if let Some(ref mut s) = session {
                                if ui.button(format!("🔗 Add Link to Page {}", target_idx + 1)).clicked() {
                                    let rect = rustpdf_geom::Rect::new(72.0, 700.0, 300.0, 730.0);
                                    let _ = s.add_link(target_idx, rect, &state.edit_link_url);
                                    close_edit = true;
                                    action = Some(ModalAction::DocumentModified);
                                }
                            }
                        }
                    }

                    ui.add_space(14.0);
                    if ui.button("Done").clicked() {
                        close_edit = true;
                    }
                });

            if close_edit {
                state.show_edit_content_dialog = false;
            }
        }

        action
    }
}
