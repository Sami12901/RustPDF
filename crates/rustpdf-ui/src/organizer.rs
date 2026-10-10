//! rustpdf-ui/organizer — Full-screen page organization grid with multi-selection.

use crate::thumbnails::ThumbnailConfig;
use egui::{
    vec2, Align, Button, Color32, ColorImage, Context, CornerRadius, Layout, Rect, ScrollArea,
    Stroke, StrokeKind, TextureHandle, TextureOptions, Ui,
};
use rustpdf_engine::DocumentSession;
use rustpdf_render::{RenderPool, RenderRequest};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct OrganizerState {
    pub selected_pages: HashSet<usize>,
    pub last_selected: Option<usize>,
    thumb_cache: HashMap<(u64, usize), TextureHandle>,
}

impl std::fmt::Debug for OrganizerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrganizerState")
            .field("selected_pages", &self.selected_pages)
            .field("last_selected", &self.last_selected)
            .finish()
    }
}

impl OrganizerState {
    pub fn select_single(&mut self, page: usize) {
        self.selected_pages.clear();
        self.selected_pages.insert(page);
        self.last_selected = Some(page);
    }

    pub fn toggle_select(&mut self, page: usize) {
        if self.selected_pages.contains(&page) {
            self.selected_pages.remove(&page);
        } else {
            self.selected_pages.insert(page);
        }
        self.last_selected = Some(page);
    }

    pub fn select_range(&mut self, to_page: usize) {
        if let Some(from) = self.last_selected {
            let start = from.min(to_page);
            let end = from.max(to_page);
            for p in start..=end {
                self.selected_pages.insert(p);
            }
        } else {
            self.select_single(to_page);
        }
    }

    pub fn select_all(&mut self, total_pages: usize) {
        self.selected_pages = (0..total_pages).collect();
    }

    pub fn deselect_all(&mut self) {
        self.selected_pages.clear();
        self.last_selected = None;
    }

    pub fn invalidate_doc(&mut self, doc_id: u64) {
        self.thumb_cache.retain(|&(d, _), _| d != doc_id);
    }

    pub fn poll_thumbnails(&mut self, ctx: &Context, pool: &RenderPool) {
        while let Some(res) = pool.try_recv() {
            let doc_id = res.tag >> 32;
            let zoom_tag = res.tag & 0xFFFF_FFFF;
            if zoom_tag == 999 && res.error.is_none() && res.width > 0 && res.height > 0 {
                let img = ColorImage::from_rgba_unmultiplied(
                    [res.width as usize, res.height as usize],
                    &res.rgba,
                );
                let handle = ctx.load_texture(
                    format!("thumb_doc{}_page_{}", doc_id, res.page_index),
                    img,
                    TextureOptions::LINEAR,
                );
                self.thumb_cache.insert((doc_id, res.page_index), handle);
            }
        }
    }
}

pub enum OrganizerAction {
    Rotate(i32),
    Delete,
    InsertBlank,
    Duplicate,
    Undo,
    Redo,
}

pub struct OrganizerComponent;

impl OrganizerComponent {
    pub fn render(
        ui: &mut Ui,
        session: &mut DocumentSession,
        state: &mut OrganizerState,
        config: &ThumbnailConfig,
        pool: &RenderPool,
        accent_color: Color32,
    ) -> Option<OrganizerAction> {
        let mut triggered_action = None;
        let total_pages = session.page_count();

        state.poll_thumbnails(ui.ctx(), pool);

        // Top Action Bar
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

            let has_selection = !state.selected_pages.is_empty();

            if ui
                .add_enabled(has_selection, Button::new("⟳ Rotate 90° CW"))
                .clicked()
            {
                triggered_action = Some(OrganizerAction::Rotate(90));
            }

            if ui
                .add_enabled(has_selection, Button::new("⟲ Rotate 90° CCW"))
                .clicked()
            {
                triggered_action = Some(OrganizerAction::Rotate(-90));
            }

            if ui
                .add_enabled(
                    has_selection && total_pages > state.selected_pages.len(),
                    Button::new("🗑 Delete"),
                )
                .on_hover_text("Delete selected pages (at least 1 page must remain)")
                .clicked()
            {
                triggered_action = Some(OrganizerAction::Delete);
            }

            if ui
                .add_enabled(has_selection, Button::new("📋 Duplicate"))
                .clicked()
            {
                triggered_action = Some(OrganizerAction::Duplicate);
            }

            if ui.button("➕ Insert Blank").clicked() {
                triggered_action = Some(OrganizerAction::InsertBlank);
            }

            ui.separator();

            if ui
                .add_enabled(session.history.can_undo(), Button::new("↩ Undo"))
                .clicked()
            {
                triggered_action = Some(OrganizerAction::Undo);
            }

            if ui
                .add_enabled(session.history.can_redo(), Button::new("↪ Redo"))
                .clicked()
            {
                triggered_action = Some(OrganizerAction::Redo);
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Deselect All").clicked() {
                    state.deselect_all();
                }
                if ui.button("Select All").clicked() {
                    state.select_all(total_pages);
                }
                ui.label(format!(
                    "{} of {} selected",
                    state.selected_pages.len(),
                    total_pages
                ));
            });
        });

        ui.separator();
        ui.add_space(10.0);

        // Thumbnail Card Grid
        let card_w = config.size;
        let spacing = 16.0 * config.density;

        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(spacing, spacing);

                for idx in 0..total_pages {
                    let page_info = &session.model.pages[idx];
                    let aspect = page_info.aspect_ratio();
                    let card_h = card_w / aspect;

                    let is_selected = state.selected_pages.contains(&idx);

                    let (rect, resp) = ui.allocate_exact_size(
                        vec2(
                            card_w,
                            card_h + if config.show_page_labels { 24.0 } else { 0.0 },
                        ),
                        egui::Sense::click(),
                    );

                    let thumb_rect = Rect::from_min_size(rect.min, vec2(card_w, card_h));

                    // Click Selection Logic (Normal, Ctrl/Cmd toggle, Shift range)
                    if resp.clicked() {
                        if ui.input(|i| i.modifiers.command || i.modifiers.ctrl) {
                            state.toggle_select(idx);
                        } else if ui.input(|i| i.modifiers.shift) {
                            state.select_range(idx);
                        } else {
                            state.select_single(idx);
                        }
                    }

                    // Shadow & Background
                    if config.show_shadow {
                        ui.painter().rect_filled(
                            thumb_rect.translate(vec2(2.0, 3.0)),
                            CornerRadius::same(config.corner_radius as u8),
                            Color32::from_black_alpha(40),
                        );
                    }

                    ui.painter().rect(
                        thumb_rect,
                        CornerRadius::same(config.corner_radius as u8),
                        Color32::WHITE,
                        if is_selected {
                            Stroke::new(config.border_width, accent_color)
                        } else {
                            Stroke::new(1.0, Color32::from_gray(180))
                        },
                        StrokeKind::Middle,
                    );

                    // Thumbnail image or request
                    let doc_id = session.id;
                    if let Some(tex) = state.thumb_cache.get(&(doc_id, idx)) {
                        ui.painter().image(
                            tex.id(),
                            thumb_rect.shrink(1.0),
                            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    } else {
                        let request_tag = (doc_id << 32) | 999;
                        pool.submit(
                            Arc::clone(&session.raw_bytes),
                            None,
                            RenderRequest {
                                page_index: idx,
                                scale: 0.25, // Fast low-res thumbnail
                                tag: request_tag,
                            },
                        );

                        ui.painter().text(
                            thumb_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}", idx + 1),
                            egui::FontId::proportional(14.0),
                            Color32::from_gray(120),
                        );
                    }

                    // Page Labels
                    if config.show_page_labels {
                        let label_rect = Rect::from_min_max(
                            egui::pos2(rect.min.x, thumb_rect.max.y + 4.0),
                            egui::pos2(rect.max.x, rect.max.y),
                        );
                        let label_text = if config.show_dimensions {
                            let (w_in, h_in) = (page_info.width / 72.0, page_info.height / 72.0);
                            format!("Page {} ({:.1}×{:.1} in)", idx + 1, w_in, h_in)
                        } else {
                            format!("Page {}", idx + 1)
                        };

                        ui.painter().text(
                            label_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label_text,
                            egui::FontId::proportional(11.0),
                            if is_selected {
                                accent_color
                            } else {
                                Color32::from_gray(150)
                            },
                        );
                    }
                }
            });
        });

        triggered_action
    }
}
