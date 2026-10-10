//! rustpdf-ui/viewer — Document viewport, continuous scrolling, zoom, and texture management.

use egui::{
    pos2, vec2, Color32, ColorImage, Context, CornerRadius, Rect, ScrollArea, Stroke,
    StrokeKind, TextureHandle, TextureOptions, Ui,
};
use rustpdf_engine::DocumentSession;
use rustpdf_render::{RenderPool, RenderRequest};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    SinglePage,
    Continuous,
    TwoUp,
}

#[derive(Clone)]
pub struct ViewerState {
    pub current_page: usize,
    pub zoom: f32, // 1.0 = 100%
    pub view_mode: ViewMode,
    pub rotation_view_deg: i32, // temporary view rotation (0, 90, 180, 270)
    texture_cache: HashMap<(usize, u64), TextureHandle>,
}

impl std::fmt::Debug for ViewerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewerState")
            .field("current_page", &self.current_page)
            .field("zoom", &self.zoom)
            .field("view_mode", &self.view_mode)
            .field("rotation_view_deg", &self.rotation_view_deg)
            .finish()
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            current_page: 0,
            zoom: 1.0,
            view_mode: ViewMode::Continuous,
            rotation_view_deg: 0,
            texture_cache: HashMap::new(),
        }
    }
}

impl ViewerState {
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.2).min(8.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.2).max(0.2);
    }

    pub fn set_zoom(&mut self, z: f32) {
        self.zoom = z.clamp(0.2, 8.0);
    }

    pub fn fit_width(&mut self, available_w: f32, page_w_pt: f32) {
        if page_w_pt > 0.0 {
            self.zoom = ((available_w - 48.0) / page_w_pt).clamp(0.2, 8.0);
        }
    }

    pub fn fit_page(&mut self, available_w: f32, available_h: f32, page_w_pt: f32, page_h_pt: f32) {
        if page_w_pt > 0.0 && page_h_pt > 0.0 {
            let zw = (available_w - 48.0) / page_w_pt;
            let zh = (available_h - 48.0) / page_h_pt;
            self.zoom = zw.min(zh).clamp(0.2, 8.0);
        }
    }

    /// Receives any finished raster pages from RenderPool and registers them with egui.
    pub fn poll_render_pool(&mut self, ctx: &Context, pool: &RenderPool) {
        while let Some(res) = pool.try_recv() {
            if res.error.is_none() && res.width > 0 && res.height > 0 {
                let img = ColorImage::from_rgba_unmultiplied(
                    [res.width as usize, res.height as usize],
                    &res.rgba,
                );
                let handle = ctx.load_texture(
                    format!("pdf_page_{}_{}", res.page_index, res.tag),
                    img,
                    TextureOptions::LINEAR,
                );
                self.texture_cache.insert((res.page_index, res.tag), handle);
            }
        }
    }

    pub fn get_texture(&self, page_index: usize, tag: u64) -> Option<&TextureHandle> {
        self.texture_cache.get(&(page_index, tag))
    }
}

pub struct ViewerComponent;

impl ViewerComponent {
    pub fn render(
        ui: &mut Ui,
        session: &DocumentSession,
        state: &mut ViewerState,
        pool: &RenderPool,
        accent_color: Color32,
    ) {
        let total_pages = session.page_count();
        if total_pages == 0 {
            ui.centered_and_justified(|ui| {
                ui.label("Document has no pages");
            });
            return;
        }

        let ctx = ui.ctx().clone();
        state.poll_render_pool(&ctx, pool);

        // Calculate current zoom generation tag
        let zoom_tag = (state.zoom * 100.0) as u64;

        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    match state.view_mode {
                        ViewMode::SinglePage => {
                            let idx = state.current_page.min(total_pages - 1);
                            Self::render_page_item(ui, session, state, pool, idx, zoom_tag, accent_color);
                        }
                        ViewMode::Continuous => {
                            for idx in 0..total_pages {
                                Self::render_page_item(ui, session, state, pool, idx, zoom_tag, accent_color);
                                ui.add_space(16.0);
                            }
                        }
                        ViewMode::TwoUp => {
                            let mut idx = 0;
                            while idx < total_pages {
                                ui.horizontal(|ui| {
                                    Self::render_page_item(ui, session, state, pool, idx, zoom_tag, accent_color);
                                    if idx + 1 < total_pages {
                                        ui.add_space(16.0);
                                        Self::render_page_item(ui, session, state, pool, idx + 1, zoom_tag, accent_color);
                                    }
                                });
                                ui.add_space(16.0);
                                idx += 2;
                            }
                        }
                    }
                });
            });
    }

    fn render_page_item(
        ui: &mut Ui,
        session: &DocumentSession,
        state: &ViewerState,
        pool: &RenderPool,
        page_index: usize,
        zoom_tag: u64,
        _accent_color: Color32,
    ) {
        let Some(page_info) = session.model.pages.get(page_index) else {
            return;
        };

        let width_px = page_info.width * state.zoom;
        let height_px = page_info.height * state.zoom;

        let (rect, _resp) = ui.allocate_exact_size(vec2(width_px, height_px), egui::Sense::click_and_drag());

        // Drop shadow & White paper background
        let shadow_rect = rect.translate(vec2(3.0, 4.0));
        ui.painter().rect_filled(
            shadow_rect,
            CornerRadius::same(2),
            Color32::from_black_alpha(50),
        );
        ui.painter().rect(
            rect,
            CornerRadius::same(2),
            Color32::WHITE,
            Stroke::new(1.0, Color32::from_gray(180)),
            StrokeKind::Middle,
        );

        if let Some(tex) = state.get_texture(page_index, zoom_tag) {
            ui.painter().image(
                tex.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // Placeholder: Request page render from pool
            pool.submit(
                Arc::clone(&session.raw_bytes),
                None,
                RenderRequest {
                    page_index,
                    scale: state.zoom * 1.5, // Render at 1.5x for crisp text
                    tag: zoom_tag,
                },
            );

            // Shimmer / loading indicator
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("Page {}", page_index + 1),
                egui::FontId::proportional(16.0),
                Color32::from_gray(120),
            );
        }

        // Live overlay rendering of document annotations (Fill & Sign items, text notes, signatures)
        let annots = session.get_annotations(page_index);
        for a in annots {
            let x0 = rect.min.x + (a.rect.min_x / page_info.width) * rect.width();
            let x1 = rect.min.x + (a.rect.max_x / page_info.width) * rect.width();
            let y0 = rect.max.y - (a.rect.max_y / page_info.height) * rect.height();
            let y1 = rect.max.y - (a.rect.min_y / page_info.height) * rect.height();
            let annot_rect = Rect::from_min_max(pos2(x0.min(x1), y0.min(y1)), pos2(x0.max(x1), y0.max(y1)));

            let color = Color32::from_rgb(
                (a.color[0] * 255.0) as u8,
                (a.color[1] * 255.0) as u8,
                (a.color[2] * 255.0) as u8,
            );

            if let Some(ref text) = a.contents {
                let bg_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 25);
                ui.painter().rect_filled(annot_rect, CornerRadius::same(3), bg_color);
                ui.painter().rect_stroke(annot_rect, CornerRadius::same(3), Stroke::new(1.0, color), StrokeKind::Inside);
                let font_size = (13.0 * state.zoom).clamp(9.0, 24.0);
                ui.painter().text(
                    annot_rect.min + vec2(4.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    text,
                    egui::FontId::proportional(font_size),
                    color,
                );
            } else if a.annot_type == rustpdf_annot::AnnotType::Circle {
                ui.painter().circle_stroke(annot_rect.center(), annot_rect.width() * 0.45, Stroke::new(2.0, color));
                ui.painter().circle_filled(annot_rect.center(), annot_rect.width() * 0.20, color);
            } else if a.annot_type == rustpdf_annot::AnnotType::Line {
                ui.painter().line_segment([pos2(annot_rect.min.x, annot_rect.center().y), pos2(annot_rect.max.x, annot_rect.center().y)], Stroke::new(2.0, color));
            }
        }
    }
}
use std::sync::Arc;
