//! rustpdf-ui/home — Welcome workspace, recent documents, and tool launcher.

use egui::{vec2, Align, Color32, CornerRadius, Layout, Rect, RichText, ScrollArea, Stroke, StrokeKind, Ui, UiBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentDocument {
    pub path: PathBuf,
    pub filename: String,
    pub page_count: usize,
    pub file_size: usize,
    pub last_opened_timestamp: u64,
    pub is_pinned: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RecentFilesManager {
    pub files: Vec<RecentDocument>,
}

impl RecentFilesManager {
    pub fn config_path() -> PathBuf {
        let mut p = if let Some(base) = std::env::var_os("APPDATA") {
            PathBuf::from(base).join("RustPdf")
        } else if let Some(base) = std::env::var_os("HOME") {
            PathBuf::from(base).join(".config").join("rustpdf")
        } else {
            PathBuf::from(".")
        };
        p.push("rustpdf_recent.json");
        p
    }

    pub fn load_persisted() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(mgr) = serde_json::from_str(&data) {
                return mgr;
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

    pub fn add_or_update(&mut self, path: &Path, page_count: usize, file_size: usize) {
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "document.pdf".to_string());

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if let Some(idx) = self.files.iter().position(|f| f.path == path) {
            let pinned = self.files[idx].is_pinned;
            self.files.remove(idx);
            self.files.insert(
                0,
                RecentDocument {
                    path: path.to_path_buf(),
                    filename,
                    page_count,
                    file_size,
                    last_opened_timestamp: timestamp,
                    is_pinned: pinned,
                },
            );
        } else {
            self.files.insert(
                0,
                RecentDocument {
                    path: path.to_path_buf(),
                    filename,
                    page_count,
                    file_size,
                    last_opened_timestamp: timestamp,
                    is_pinned: false,
                },
            );
        }

        // Cap to 50 recent files
        if self.files.len() > 50 {
            self.files.truncate(50);
        }

        let _ = self.save_persisted();
    }

    pub fn toggle_pin(&mut self, index: usize) {
        if let Some(item) = self.files.get_mut(index) {
            item.is_pinned = !item.is_pinned;
            let _ = self.save_persisted();
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.files.len() {
            self.files.remove(index);
            let _ = self.save_persisted();
        }
    }
}

pub enum HomeAction {
    OpenPdf,
    OpenFile(PathBuf),
    NewBlank,
    CombineDialog,
    SplitDialog,
    AppearanceSettings,
    ThumbnailSettings,
    OrganizePages,
    AddComments,
    PrepareForm,
    EditPdf,
    ProtectPdf,
}

pub struct HomeView;

impl HomeView {
    pub fn render(
        ui: &mut Ui,
        recent: &mut RecentFilesManager,
        accent_color: Color32,
    ) -> Option<HomeAction> {
        let mut action = None;

        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(20.0);

            // Hero Brand Header with In-App Brand Identity
            ui.vertical_centered(|ui| {
                let (response, painter) = ui.allocate_painter(vec2(64.0, 72.0), egui::Sense::hover());
                crate::icons::Icons::draw_logo(&painter, response.rect, accent_color);

                ui.add_space(8.0);
                ui.heading(
                    RichText::new("RustPdf")
                        .size(30.0)
                        .strong()
                        .color(accent_color),
                );
                ui.label(
                    RichText::new("Professional Open-Source PDF Workbench")
                        .size(14.0)
                        .color(Color32::from_gray(160)),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                    ui.label(RichText::new("100% Offline").size(11.0).color(Color32::from_rgb(50, 190, 130)));
                    ui.label(RichText::new("•").size(10.0).color(Color32::from_gray(100)));
                    ui.label(RichText::new("Zero Telemetry").size(11.0).color(Color32::from_rgb(90, 170, 255)));
                    ui.label(RichText::new("•").size(10.0).color(Color32::from_gray(100)));
                    ui.label(RichText::new("Pure Memory-Safe Rust").size(11.0).color(Color32::from_rgb(255, 150, 80)));
                });
            });

            ui.add_space(24.0);

            // "Recommended tools" Section matching Screenshot 3
            ui.label(RichText::new("Recommended tools").size(17.5).strong().color(Color32::from_gray(230)));
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(12.0, 12.0);

                let card_w = 185.0;
                let card_h = 120.0;

                // Helper to render tool cards with icon, bullet subtitle, and 'Use now' link
                let draw_tool_card = |ui: &mut Ui, icon: crate::icons::IconKind, title: &str, subtitle: &str| -> bool {
                    let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), egui::Sense::click());
                    let is_hovered = resp.hovered();
                    let is_dark = ui.visuals().dark_mode;

                    let bg = if is_dark {
                        if is_hovered { Color32::from_rgb(36, 39, 49) } else { Color32::from_rgb(26, 28, 35) }
                    } else {
                        if is_hovered { Color32::from_rgb(248, 250, 253) } else { Color32::from_rgb(255, 255, 255) }
                    };

                    let border = if is_hovered {
                        accent_color
                    } else if is_dark {
                        Color32::from_rgb(46, 50, 64)
                    } else {
                        Color32::from_rgb(222, 226, 235)
                    };

                    ui.painter().rect(
                        rect,
                        CornerRadius::same(8),
                        bg,
                        Stroke::new(1.0, border),
                        StrokeKind::Middle,
                    );

                    // Header row: Icon + Bold Title
                    let icon_rect = Rect::from_center_size(
                        egui::pos2(rect.min.x + 22.0, rect.min.y + 24.0),
                        vec2(20.0, 20.0),
                    );
                    crate::icons::Icons::draw_icon(ui.painter(), icon_rect, icon, icon.default_color());

                    let title_rect = Rect::from_min_max(
                        egui::pos2(rect.min.x + 40.0, rect.min.y + 12.0),
                        egui::pos2(rect.max.x - 10.0, rect.min.y + 36.0),
                    );
                    let mut title_child = ui.new_child(UiBuilder::new().max_rect(title_rect).layout(Layout::left_to_right(Align::Center)));
                    title_child.label(RichText::new(title).strong().size(13.0));

                    // Middle: Subtitle bullet list
                    let sub_rect = Rect::from_min_max(
                        egui::pos2(rect.min.x + 14.0, rect.min.y + 44.0),
                        egui::pos2(rect.max.x - 12.0, rect.max.y - 34.0),
                    );
                    let mut sub_child = ui.new_child(UiBuilder::new().max_rect(sub_rect).layout(Layout::top_down(Align::Min)));
                    sub_child.label(RichText::new(subtitle).size(10.0).color(Color32::from_gray(140)));

                    // Bottom: "Use now" signature text button
                    let use_now_rect = Rect::from_min_max(
                        egui::pos2(rect.min.x + 14.0, rect.max.y - 28.0),
                        egui::pos2(rect.min.x + 80.0, rect.max.y - 8.0),
                    );
                    let mut link_child = ui.new_child(UiBuilder::new().max_rect(use_now_rect).layout(Layout::left_to_right(Align::Center)));
                    let link_color = Color32::from_rgb(37, 99, 235);
                    let link_resp = link_child.link(RichText::new("Use now").size(12.0).strong().color(link_color));

                    resp.clicked() || link_resp.clicked()
                };

                // Card 1: Organize pages
                if draw_tool_card(ui, crate::icons::IconKind::OrganizePages, "Organize pages", "Page grid · Rotate · Delete") {
                    action = Some(HomeAction::OrganizePages);
                }

                // Card 2: Add comments
                if draw_tool_card(ui, crate::icons::IconKind::AddComments, "Add comments", "Sticky note · Highlight text · Underline") {
                    action = Some(HomeAction::AddComments);
                }

                // Card 3: Prepare a form
                if draw_tool_card(ui, crate::icons::IconKind::PrepareForm, "Prepare a form", "Field list · Detect form fields · Clear form") {
                    action = Some(HomeAction::PrepareForm);
                }

                // Card 4: Edit a PDF
                if draw_tool_card(ui, crate::icons::IconKind::EditPdf, "Edit a PDF", "Rotate pages · Insert pages · Delete pages") {
                    action = Some(HomeAction::EditPdf);
                }

                // Card 5: Protect a PDF
                if draw_tool_card(ui, crate::icons::IconKind::ProtectPdf, "Protect a PDF", "Protect with password · Remove hidden information") {
                    action = Some(HomeAction::ProtectPdf);
                }

                // Card 6: Open file (Solid action card matching Screenshot 3)
                let (open_rect, open_resp) = ui.allocate_exact_size(vec2(140.0, card_h), egui::Sense::click());
                let is_dark = ui.visuals().dark_mode;
                let open_bg = if is_dark {
                    if open_resp.hovered() { Color32::from_rgb(44, 48, 60) } else { Color32::from_rgb(32, 34, 42) }
                } else {
                    if open_resp.hovered() { Color32::from_rgb(235, 238, 244) } else { Color32::from_rgb(243, 244, 248) }
                };
                ui.painter().rect(
                    open_rect,
                    CornerRadius::same(8),
                    open_bg,
                    Stroke::new(1.0, if open_resp.hovered() { accent_color } else { Color32::from_gray(55) }),
                    StrokeKind::Middle,
                );
                let folder_rect = Rect::from_center_size(
                    egui::pos2(open_rect.center().x, open_rect.min.y + 44.0),
                    vec2(30.0, 30.0),
                );
                crate::icons::Icons::draw_icon(ui.painter(), folder_rect, crate::icons::IconKind::FolderOpen, Color32::from_gray(160));
                let label_rect = Rect::from_min_max(
                    egui::pos2(open_rect.min.x, open_rect.min.y + 68.0),
                    egui::pos2(open_rect.max.x, open_rect.max.y - 12.0),
                );
                let mut label_child = ui.new_child(UiBuilder::new().max_rect(label_rect).layout(Layout::centered_and_justified(egui::Direction::LeftToRight)));
                label_child.label(RichText::new("Open file").strong().size(13.0));

                if open_resp.clicked() {
                    action = Some(HomeAction::OpenPdf);
                }
            });

            ui.add_space(32.0);

            // Recent Documents Section Header matching Screenshot 3
            ui.horizontal(|ui| {
                ui.label(RichText::new("Recent").size(17.5).strong().color(Color32::from_gray(230)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if !recent.files.is_empty() && ui.button("Clear History").clicked() {
                        recent.files.clear();
                        let _ = recent.save_persisted();
                    }
                });
            });

            ui.separator();
            ui.add_space(8.0);

            if recent.files.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new("No recent documents yet.").size(14.0).color(Color32::from_gray(140)));
                    ui.label(RichText::new("Drag and drop any PDF file here to start.").size(12.0).color(Color32::from_gray(110)));
                    ui.add_space(40.0);
                });
            } else {
                let mut to_toggle_pin = None;
                let mut to_remove = None;

                for (idx, doc) in recent.files.iter().enumerate() {
                    let (row_rect, row_resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), egui::Sense::click());

                    let row_bg = if row_resp.hovered() {
                        Color32::from_rgb(32, 35, 42)
                    } else {
                        Color32::from_rgb(22, 23, 27)
                    };

                    ui.painter().rect(row_rect, CornerRadius::same(6), row_bg, Stroke::new(1.0, Color32::from_rgb(40, 42, 50)), StrokeKind::Middle);

                    if row_resp.clicked() {
                        action = Some(HomeAction::OpenFile(doc.path.clone()));
                    }

                    let mut child = ui.new_child(UiBuilder::new().max_rect(row_rect.shrink(8.0)).layout(Layout::left_to_right(Align::Center)));
                    child.label(RichText::new("📄").size(18.0));
                    child.add_space(8.0);

                    child.vertical(|ui| {
                        ui.label(RichText::new(&doc.filename).strong().size(13.0));
                        let size_kb = doc.file_size / 1024;
                        ui.label(
                            RichText::new(format!("{} pages • {} KB • {}", doc.page_count, size_kb, doc.path.display()))
                                .size(11.0)
                                .color(Color32::from_gray(140)),
                        );
                    });

                    child.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("✖").on_hover_text("Remove from history").clicked() {
                            to_remove = Some(idx);
                        }

                        let pin_label = if doc.is_pinned { "📌" } else { "📍" };
                        if ui.button(pin_label).on_hover_text("Toggle Pin").clicked() {
                            to_toggle_pin = Some(idx);
                        }
                    });

                    ui.add_space(4.0);
                }

                if let Some(idx) = to_toggle_pin {
                    recent.toggle_pin(idx);
                }
                if let Some(idx) = to_remove {
                    recent.remove(idx);
                }
            }

            ui.add_space(30.0);
        });

        action
    }
}
