//! rustpdf-ui/viewer — Document viewport, continuous scrolling, zoom, and interactive in-place text & image editing.

use egui::{
    pos2, vec2, Align2, Color32, ColorImage, Context, CornerRadius, Id, Key, Order, Rect,
    ScrollArea, Stroke, StrokeKind, TextEdit, TextureHandle, TextureOptions, Ui,
};
use rustpdf_engine::DocumentSession;
use rustpdf_fonts::StandardFont;
use rustpdf_render::{RenderPool, RenderRequest};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    SinglePage,
    Continuous,
    TwoUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StampKind {
    Checkmark,
    Cross,
    Dot,
    Line,
    Date,
    Signature,
}

impl StampKind {
    pub fn label(&self) -> &'static str {
        match self {
            StampKind::Checkmark => "Checkmark",
            StampKind::Cross => "Cross",
            StampKind::Dot => "Dot",
            StampKind::Line => "Line",
            StampKind::Date => "Date",
            StampKind::Signature => "Signature",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditTool {
    Select,   // Select & Edit existing text bounding boxes
    Hand,     // Hand / Pan document
    AddText,  // Click on page to place a new text box
    AddImage, // Click on page to embed an image
    Draw,     // Ink / Drawing
    Stamp(StampKind), // Click on page to place a Fill & Sign stamp
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedTextEdit {
    pub page_index: usize,
    pub block_id: usize,
    pub target_block: rustpdf_edit::PageTextBlock,
    pub original_text: String,
    pub current_text: String,
    pub font_size: f32,
    pub screen_rect: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewTextPlacement {
    pub page_index: usize,
    pub pt_x: f32,
    pub pt_y: f32,
    pub text: String,
    pub font_size: f32,
}

#[derive(Clone)]
pub struct ViewerState {
    pub active_doc_id: Option<u64>,
    pub current_page: usize,
    pub zoom: f32, // 1.0 = 100%
    pub view_mode: ViewMode,
    pub rotation_view_deg: i32,
    pub is_edit_mode: bool,
    pub active_tool: EditTool,
    pub selected_edit: Option<SelectedTextEdit>,
    pub new_text_placement: Option<NewTextPlacement>,
    pub text_blocks_cache: HashMap<(u64, usize), Vec<rustpdf_edit::PageTextBlock>>,
    texture_cache: HashMap<(u64, usize, u64), TextureHandle>,
}

impl std::fmt::Debug for ViewerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewerState")
            .field("active_doc_id", &self.active_doc_id)
            .field("current_page", &self.current_page)
            .field("zoom", &self.zoom)
            .field("view_mode", &self.view_mode)
            .field("rotation_view_deg", &self.rotation_view_deg)
            .field("is_edit_mode", &self.is_edit_mode)
            .field("active_tool", &self.active_tool)
            .finish()
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            active_doc_id: None,
            current_page: 0,
            zoom: 1.0,
            view_mode: ViewMode::Continuous,
            rotation_view_deg: 0,
            is_edit_mode: false,
            active_tool: EditTool::Select,
            selected_edit: None,
            new_text_placement: None,
            text_blocks_cache: HashMap::new(),
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

    pub fn invalidate_page_texture(&mut self, doc_id: u64, page_index: usize) {
        self.texture_cache
            .retain(|&(d, p, _), _| !(d == doc_id && p == page_index));
        self.text_blocks_cache.remove(&(doc_id, page_index));
    }

    pub fn invalidate_doc(&mut self, doc_id: u64) {
        self.texture_cache.retain(|&(d, _, _), _| d != doc_id);
        self.text_blocks_cache.retain(|&(d, _), _| d != doc_id);
        if self.active_doc_id == Some(doc_id) {
            self.active_doc_id = None;
            self.selected_edit = None;
            self.new_text_placement = None;
        }
    }

    /// Receives any finished raster pages from RenderPool and registers them with egui.
    pub fn poll_render_pool(&mut self, ctx: &Context, pool: &RenderPool) {
        while let Some(res) = pool.try_recv() {
            let doc_id = res.tag >> 32;
            let zoom_tag = res.tag & 0xFFFF_FFFF;
            // Ignore organizer thumbnail tags (zoom_tag == 999)
            if zoom_tag != 999 && res.error.is_none() && res.width > 0 && res.height > 0 {
                let img = ColorImage::from_rgba_unmultiplied(
                    [res.width as usize, res.height as usize],
                    &res.rgba,
                );
                let handle = ctx.load_texture(
                    format!("pdf_page_doc{}_{}_{}", doc_id, res.page_index, zoom_tag),
                    img,
                    TextureOptions::LINEAR,
                );
                self.texture_cache
                    .insert((doc_id, res.page_index, zoom_tag), handle);
            }
        }
    }

    pub fn get_texture(
        &self,
        doc_id: u64,
        page_index: usize,
        zoom_tag: u64,
    ) -> Option<&TextureHandle> {
        self.texture_cache.get(&(doc_id, page_index, zoom_tag))
    }
}

pub struct ViewerComponent;

impl ViewerComponent {
    pub fn render(
        ui: &mut Ui,
        session: &mut DocumentSession,
        state: &mut ViewerState,
        pool: &RenderPool,
        accent_color: Color32,
        search_state: Option<&crate::search::SearchState>,
    ) {
        let doc_id = session.id;
        if state.active_doc_id != Some(doc_id) {
            state.active_doc_id = Some(doc_id);
            state.current_page = 0;
            state.selected_edit = None;
            state.new_text_placement = None;
        }

        let total_pages = session.page_count();
        if total_pages == 0 {
            ui.centered_and_justified(|ui| {
                ui.label("Document has no pages");
            });
            return;
        }

        let ctx = ui.ctx().clone();
        state.poll_render_pool(&ctx, pool);

        let zoom_tag = (state.zoom * 100.0) as u64;

        // 1. Floating vertical toolbar on the left side of the canvas (matching reference app)
        Self::render_floating_toolbar(ui, state, accent_color);

        // 2. Document Viewport with Page Items
        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| match state.view_mode {
                    ViewMode::SinglePage => {
                        let idx = state.current_page.min(total_pages - 1);
                        Self::render_page_item(
                            ui,
                            session,
                            state,
                            pool,
                            idx,
                            zoom_tag,
                            accent_color,
                            search_state,
                        );
                    }
                    ViewMode::Continuous => {
                        for idx in 0..total_pages {
                            Self::render_page_item(
                                ui,
                                session,
                                state,
                                pool,
                                idx,
                                zoom_tag,
                                accent_color,
                                search_state,
                            );
                            ui.add_space(16.0);
                        }
                    }
                    ViewMode::TwoUp => {
                        let mut idx = 0;
                        while idx < total_pages {
                            ui.horizontal(|ui| {
                                Self::render_page_item(
                                    ui,
                                    session,
                                    state,
                                    pool,
                                    idx,
                                    zoom_tag,
                                    accent_color,
                                    search_state,
                                );
                                if idx + 1 < total_pages {
                                    ui.add_space(16.0);
                                    Self::render_page_item(
                                        ui,
                                        session,
                                        state,
                                        pool,
                                        idx + 1,
                                        zoom_tag,
                                        accent_color,
                                        search_state,
                                    );
                                }
                            });
                            ui.add_space(16.0);
                            idx += 2;
                        }
                    }
                });
            });
    }

    /// Renders the floating vertical tool palette seen in the reference application screenshot.
    fn render_floating_toolbar(ui: &mut Ui, state: &mut ViewerState, accent_color: Color32) {
        let is_dark = ui.visuals().dark_mode;
        let bg_color = if is_dark {
            Color32::from_rgba_unmultiplied(30, 30, 35, 235)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 240)
        };
        let border_stroke = Stroke::new(
            1.0,
            if is_dark {
                Color32::from_gray(70)
            } else {
                Color32::from_gray(210)
            },
        );

        egui::Area::new(Id::new("viewer_floating_tool_palette"))
            .order(Order::Foreground)
            .anchor(Align2::LEFT_TOP, vec2(16.0, 100.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style())
                    .fill(bg_color)
                    .stroke(border_stroke)
                    .corner_radius(CornerRadius::same(8))
                    .shadow(egui::Shadow {
                        offset: [0, 4],
                        blur: 12,
                        spread: 0,
                        color: Color32::from_black_alpha(45),
                    })
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 4.0);

                        // Tool 1: Pointer (Select & Edit Text/Images)
                        let is_select = state.is_edit_mode && state.active_tool == EditTool::Select;
                        if ui
                            .selectable_label(is_select, " ↖ ")
                            .on_hover_text("Select & Edit Text / Image Blocks")
                            .clicked()
                        {
                            state.is_edit_mode = true;
                            state.active_tool = EditTool::Select;
                        }

                        // Tool 2: Hand (Pan Document)
                        let is_hand = state.active_tool == EditTool::Hand;
                        if ui
                            .selectable_label(is_hand, " ✋ ")
                            .on_hover_text("Hand / Pan Document")
                            .clicked()
                        {
                            state.active_tool = EditTool::Hand;
                            state.selected_edit = None;
                        }

                        ui.separator();

                        // Tool 3: Add Text Box (T+)
                        let is_add_text =
                            state.is_edit_mode && state.active_tool == EditTool::AddText;
                        if ui
                            .selectable_label(is_add_text, " T+ ")
                            .on_hover_text("Add New Text Box (Click on page)")
                            .clicked()
                        {
                            state.is_edit_mode = true;
                            state.active_tool = EditTool::AddText;
                        }

                        // Tool 4: Add Image
                        let is_add_img =
                            state.is_edit_mode && state.active_tool == EditTool::AddImage;
                        if ui
                            .selectable_label(is_add_img, " 🖼 ")
                            .on_hover_text("Insert Image (Click on page)")
                            .clicked()
                        {
                            state.is_edit_mode = true;
                            state.active_tool = EditTool::AddImage;
                        }

                        // Tool 5: Draw / Pen
                        let is_draw = state.active_tool == EditTool::Draw;
                        if ui
                            .selectable_label(is_draw, " ✏ ")
                            .on_hover_text("Pencil / Draw Annotation")
                            .clicked()
                        {
                            state.active_tool = EditTool::Draw;
                        }

                        // Tool 6: Fill & Sign Stamp (Checkmark)
                        let is_stamp = matches!(state.active_tool, EditTool::Stamp(_));
                        if ui
                            .selectable_label(is_stamp, " ✔ ")
                            .on_hover_text("Stamp Symbol (Click on page)")
                            .clicked()
                        {
                            state.is_edit_mode = true;
                            state.active_tool = EditTool::Stamp(StampKind::Checkmark);
                        }

                        ui.separator();

                        // Toggle Edit Mode ON / OFF
                        let edit_btn_label = if state.is_edit_mode {
                            "✎ ON"
                        } else {
                            "✎ Edit"
                        };
                        let edit_color = if state.is_edit_mode {
                            accent_color
                        } else {
                            Color32::from_gray(120)
                        };
                        if ui
                            .button(
                                egui::RichText::new(edit_btn_label)
                                    .color(edit_color)
                                    .strong(),
                            )
                            .on_hover_text("Toggle Interactive In-Page Content Bounding Boxes")
                            .clicked()
                        {
                            state.is_edit_mode = !state.is_edit_mode;
                            if !state.is_edit_mode {
                                state.selected_edit = None;
                                state.new_text_placement = None;
                            }
                        }
                    });
            });
    }

    fn render_page_item(
        ui: &mut Ui,
        session: &mut DocumentSession,
        state: &mut ViewerState,
        pool: &RenderPool,
        page_index: usize,
        zoom_tag: u64,
        _accent_color: Color32,
        search_state: Option<&crate::search::SearchState>,
    ) {
        let doc_id = session.id;
        let Some(page_info) = session.model.pages.get(page_index).cloned() else {
            return;
        };

        let width_px = page_info.width * state.zoom;
        let height_px = page_info.height * state.zoom;

        let (rect, resp) =
            ui.allocate_exact_size(vec2(width_px, height_px), egui::Sense::click_and_drag());

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

        let request_tag = (doc_id << 32) | (zoom_tag & 0xFFFF_FFFF);

        // Rasterized Page Texture
        if let Some(tex) = state.get_texture(doc_id, page_index, zoom_tag) {
            ui.painter().image(
                tex.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            pool.submit(
                Arc::clone(&session.raw_bytes),
                None,
                RenderRequest {
                    page_index,
                    scale: state.zoom * 1.5,
                    tag: request_tag,
                },
            );

            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                format!("Page {}", page_index + 1),
                egui::FontId::proportional(16.0),
                Color32::from_gray(120),
            );
        }

        // =========================================================================
        // SEARCH MATCH HIGHLIGHTS (ACROSS DOCUMENT)
        // =========================================================================
        if let Some(search) = search_state {
            if search.is_open {
                for (hit_idx, hit) in search.hits.iter().enumerate() {
                    if hit.page_index == page_index {
                        let x0 = rect.min.x + (hit.rect[0] / page_info.width) * rect.width();
                        let y0 = rect.max.y
                            - ((hit.rect[1] + hit.rect[3]) / page_info.height) * rect.height();
                        let w = (hit.rect[2] / page_info.width) * rect.width();
                        let h = (hit.rect[3] / page_info.height) * rect.height();
                        let hit_screen_rect =
                            Rect::from_min_size(pos2(x0, y0), vec2(w.max(6.0), h.max(8.0)));

                        let is_active = hit_idx == search.active_match_index;
                        let fill = if is_active {
                            Color32::from_rgba_unmultiplied(255, 152, 0, 160)
                        } else {
                            Color32::from_rgba_unmultiplied(255, 235, 59, 130)
                        };
                        let stroke = if is_active {
                            Stroke::new(1.5, Color32::from_rgb(230, 81, 0))
                        } else {
                            Stroke::NONE
                        };

                        ui.painter()
                            .rect_filled(hit_screen_rect, CornerRadius::same(2), fill);
                        if stroke != Stroke::NONE {
                            ui.painter().rect_stroke(
                                hit_screen_rect,
                                CornerRadius::same(2),
                                stroke,
                                StrokeKind::Inside,
                            );
                        }
                    }
                }
            }
        }

        // =========================================================================
        // INTERACTIVE IN-PAGE "EDIT TEXT & IMAGES" BOUNDING BOXES (MATCHES SCREENSHOT 2)
        // =========================================================================
        if state.is_edit_mode {
            // Retrieve or cache text blocks for this page
            if !state.text_blocks_cache.contains_key(&(doc_id, page_index)) {
                let blocks = session.get_page_text_blocks(page_index);
                state.text_blocks_cache.insert((doc_id, page_index), blocks);
            }

            let blocks = state
                .text_blocks_cache
                .get(&(doc_id, page_index))
                .cloned()
                .unwrap_or_default();
            let mut clicked_block: Option<SelectedTextEdit> = None;
            let mut apply_replacement: Option<(rustpdf_edit::PageTextBlock, String)> = None;
            let mut delete_text: Option<rustpdf_edit::PageTextBlock> = None;

            for block in &blocks {
                // Map PDF points (bottom-left origin) to screen viewport pixels
                let x0 = rect.min.x + (block.rect[0] / page_info.width) * rect.width();
                let y0 = rect.max.y
                    - ((block.rect[1] + block.rect[3]) / page_info.height) * rect.height();
                let w = (block.rect[2] / page_info.width) * rect.width();
                let h = (block.rect[3] / page_info.height) * rect.height();

                let block_screen_rect =
                    Rect::from_min_size(pos2(x0, y0), vec2(w.max(16.0), h.max(12.0)));

                let is_selected = state.selected_edit.as_ref().map_or(false, |s| {
                    s.page_index == page_index && s.block_id == block.id
                });
                let mouse_pos = ui.input(|i| i.pointer.hover_pos());
                let is_hovered = mouse_pos.map_or(false, |p| block_screen_rect.contains(p));

                // Bounding box colors (clean blue outline matching screenshot 2)
                let box_stroke = if is_selected {
                    Stroke::new(2.0, Color32::from_rgb(0, 102, 204))
                } else if is_hovered {
                    Stroke::new(1.5, Color32::from_rgb(0, 120, 215))
                } else {
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 120, 215, 175))
                };

                let box_fill = if is_selected {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 230)
                } else if is_hovered {
                    Color32::from_rgba_unmultiplied(0, 120, 215, 25)
                } else {
                    Color32::from_rgba_unmultiplied(0, 120, 215, 8)
                };

                // Draw bounding box
                ui.painter()
                    .rect_filled(block_screen_rect, CornerRadius::same(2), box_fill);
                ui.painter().rect_stroke(
                    block_screen_rect,
                    CornerRadius::same(2),
                    box_stroke,
                    StrokeKind::Inside,
                );

                // Draw small corner selection handles
                let handle_sz = 3.5;
                let handle_color = Color32::from_rgb(0, 120, 215);
                for corner in [
                    block_screen_rect.left_top(),
                    block_screen_rect.right_top(),
                    block_screen_rect.left_bottom(),
                    block_screen_rect.right_bottom(),
                ] {
                    ui.painter().rect_filled(
                        Rect::from_center_size(corner, vec2(handle_sz, handle_sz)),
                        CornerRadius::ZERO,
                        handle_color,
                    );
                }

                // If hovered and user clicks, select this block for editing
                if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                    clicked_block = Some(SelectedTextEdit {
                        page_index,
                        block_id: block.id,
                        target_block: block.clone(),
                        original_text: block.text.clone(),
                        current_text: block.text.clone(),
                        font_size: block.font_size,
                        screen_rect: block_screen_rect,
                    });
                }
            }

            if let Some(sel) = clicked_block {
                state.selected_edit = Some(sel);
            }

            // =====================================================================
            // IN-PLACE INLINE TEXT EDITOR WIDGET (OVER SELECTED TEXT BLOCK)
            // =====================================================================
            if let Some(ref mut edit) = state.selected_edit {
                if edit.page_index == page_index {
                    let mut close_edit = false;

                    egui::Area::new(Id::new(format!(
                        "inline_text_edit_{}_{}",
                        page_index, edit.block_id
                    )))
                    .order(Order::Foreground)
                    .fixed_pos(edit.screen_rect.min - vec2(0.0, 36.0))
                    .show(ui.ctx(), |ui| {
                        egui::Frame::popup(ui.style())
                            .fill(Color32::from_rgb(250, 250, 252))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0, 120, 215)))
                            .corner_radius(CornerRadius::same(6))
                            .shadow(egui::Shadow {
                                offset: [0, 4],
                                blur: 10,
                                spread: 0,
                                color: Color32::from_black_alpha(60),
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);

                                    // Text Edit Input
                                    let edit_resp = ui.add(
                                        TextEdit::singleline(&mut edit.current_text)
                                            .desired_width(
                                                (edit.screen_rect.width() + 60.0)
                                                    .clamp(160.0, 400.0),
                                            )
                                            .font(egui::TextStyle::Body),
                                    );

                                    if edit_resp.lost_focus()
                                        && ui.input(|i| i.key_pressed(Key::Enter))
                                    {
                                        apply_replacement = Some((
                                            edit.target_block.clone(),
                                            edit.current_text.clone(),
                                        ));
                                        close_edit = true;
                                    }

                                    // Apply Button
                                    if ui
                                        .button(
                                            egui::RichText::new("✓ Save")
                                                .color(Color32::from_rgb(16, 149, 193))
                                                .strong(),
                                        )
                                        .on_hover_text("Apply changes to PDF")
                                        .clicked()
                                    {
                                        apply_replacement = Some((
                                            edit.target_block.clone(),
                                            edit.current_text.clone(),
                                        ));
                                        close_edit = true;
                                    }

                                    // Delete Button
                                    if ui.button("🗑").on_hover_text("Delete this text").clicked()
                                    {
                                        delete_text = Some(edit.target_block.clone());
                                        close_edit = true;
                                    }

                                    // Close Button
                                    if ui.button("✕").on_hover_text("Cancel").clicked() {
                                        close_edit = true;
                                    }
                                });
                            });
                    });

                    if close_edit {
                        state.selected_edit = None;
                    }
                }
            }

            // Commit In-Place Replacements or Deletions
            if let Some((target_block, repl)) = apply_replacement {
                if target_block.text != repl {
                    let _ = session.replace_text_block(page_index, &target_block, &repl);
                    state.invalidate_page_texture(doc_id, page_index);
                    state.text_blocks_cache.retain(|&(d, _), _| d != doc_id);
                }
            }
            if let Some(target_block) = delete_text {
                let _ = session.replace_text_block(page_index, &target_block, "");
                state.invalidate_page_texture(doc_id, page_index);
                state.text_blocks_cache.retain(|&(d, _), _| d != doc_id);
            }

            // =====================================================================
            // TOOL: ADD NEW TEXT BOX BY CLICKING ANYWHERE ON THE PAGE
            // =====================================================================
            if state.active_tool == EditTool::AddText && resp.clicked() {
                if let Some(pointer_pos) = resp.interact_pointer_pos() {
                    let pt_x = ((pointer_pos.x - rect.min.x) / rect.width()) * page_info.width;
                    let pt_y = ((rect.max.y - pointer_pos.y) / rect.height()) * page_info.height;

                    state.new_text_placement = Some(NewTextPlacement {
                        page_index,
                        pt_x,
                        pt_y,
                        text: "New Text".to_string(),
                        font_size: 14.0,
                    });
                }
            }

            // =====================================================================
            // TOOL: ADD IMAGE BY CLICKING ON PAGE
            // =====================================================================
            if state.active_tool == EditTool::AddImage && resp.clicked() {
                if let Some(pointer_pos) = resp.interact_pointer_pos() {
                    let pt_x = ((pointer_pos.x - rect.min.x) / rect.width()) * page_info.width;
                    let pt_y = ((rect.max.y - pointer_pos.y) / rect.height()) * page_info.height;

                    if let Some(img_path) = rfd::FileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    {
                        if let Ok(bytes) = std::fs::read(&img_path) {
                            let _ = session
                                .insert_image_to_page(page_index, &bytes, pt_x, pt_y, 160.0, 120.0);
                            state.invalidate_page_texture(doc_id, page_index);
                            state.active_tool = EditTool::Select;
                        }
                    }
                }
            }

            // =====================================================================
            // TOOL: FILL & SIGN STAMP BY CLICKING ANYWHERE ON PAGE
            // =====================================================================
            if let EditTool::Stamp(stamp_kind) = state.active_tool {
                if resp.clicked() {
                    if let Some(pointer_pos) = resp.interact_pointer_pos() {
                        let pt_x = ((pointer_pos.x - rect.min.x) / rect.width()) * page_info.width;
                        let pt_y = ((rect.max.y - pointer_pos.y) / rect.height()) * page_info.height;

                        let (annot_type, contents, annot_rect) = match stamp_kind {
                            StampKind::Checkmark => (
                                rustpdf_annot::AnnotType::Text,
                                "✓".to_string(),
                                rustpdf_geom::Rect::new(pt_x - 12.0, pt_y - 12.0, pt_x + 12.0, pt_y + 12.0),
                            ),
                            StampKind::Cross => (
                                rustpdf_annot::AnnotType::Text,
                                "✕".to_string(),
                                rustpdf_geom::Rect::new(pt_x - 12.0, pt_y - 12.0, pt_x + 12.0, pt_y + 12.0),
                            ),
                            StampKind::Dot => (
                                rustpdf_annot::AnnotType::Circle,
                                "●".to_string(),
                                rustpdf_geom::Rect::new(pt_x - 10.0, pt_y - 10.0, pt_x + 10.0, pt_y + 10.0),
                            ),
                            StampKind::Line => (
                                rustpdf_annot::AnnotType::Line,
                                "—".to_string(),
                                rustpdf_geom::Rect::new(pt_x - 40.0, pt_y, pt_x + 40.0, pt_y),
                            ),
                            StampKind::Date => (
                                rustpdf_annot::AnnotType::Text,
                                chrono::Local::now().format("%Y-%m-%d").to_string(),
                                rustpdf_geom::Rect::new(pt_x, pt_y - 8.0, pt_x + 100.0, pt_y + 16.0),
                            ),
                            StampKind::Signature => (
                                rustpdf_annot::AnnotType::Text,
                                "Signature".to_string(),
                                rustpdf_geom::Rect::new(pt_x, pt_y - 12.0, pt_x + 150.0, pt_y + 20.0),
                            ),
                        };

                        let annot = rustpdf_annot::Annotation {
                            id: None,
                            page_index,
                            annot_type,
                            rect: annot_rect,
                            color: [0.55, 0.36, 0.96],
                            contents: Some(contents),
                            author: Some("Fill & Sign".to_string()),
                            date: Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()),
                            ink_paths: None,
                        };
                        let _ = session.add_annotation(&annot);
                        state.invalidate_page_texture(doc_id, page_index);
                    }
                }
            }

            // Render New Text Box Modal/Popup if active
            if let Some(ref mut placement) = state.new_text_placement {
                if placement.page_index == page_index {
                    let mut dismiss_new = false;
                    let mut commit_new = false;

                    let screen_x = rect.min.x + (placement.pt_x / page_info.width) * rect.width();
                    let screen_y = rect.max.y - (placement.pt_y / page_info.height) * rect.height();

                    egui::Area::new(Id::new(format!("new_text_box_pos_{}", page_index)))
                        .order(Order::Foreground)
                        .fixed_pos(pos2(screen_x, screen_y))
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .fill(Color32::from_rgb(255, 255, 255))
                                .stroke(Stroke::new(1.5, Color32::from_rgb(0, 120, 215)))
                                .corner_radius(CornerRadius::same(6))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            TextEdit::singleline(&mut placement.text)
                                                .desired_width(180.0),
                                        );
                                        if ui
                                            .button(
                                                egui::RichText::new("+ Add")
                                                    .strong()
                                                    .color(Color32::from_rgb(16, 149, 193)),
                                            )
                                            .clicked()
                                        {
                                            commit_new = true;
                                        }
                                        if ui.button("✕").clicked() {
                                            dismiss_new = true;
                                        }
                                    });
                                });
                        });

                    if commit_new {
                        let _ = session.add_text_to_page(
                            page_index,
                            &placement.text,
                            placement.pt_x,
                            placement.pt_y,
                            StandardFont::Helvetica,
                            placement.font_size,
                            [0.0, 0.0, 0.0],
                        );
                        state.invalidate_page_texture(doc_id, page_index);
                        dismiss_new = true;
                    }

                    if dismiss_new {
                        state.new_text_placement = None;
                    }
                }
            }
        }

        // Live overlay rendering of document annotations (Fill & Sign items, text notes, signatures)
        let annots = session.get_annotations(page_index);
        for a in annots {
            let x0 = rect.min.x + (a.rect.min_x / page_info.width) * rect.width();
            let x1 = rect.min.x + (a.rect.max_x / page_info.width) * rect.width();
            let y0 = rect.max.y - (a.rect.max_y / page_info.height) * rect.height();
            let y1 = rect.max.y - (a.rect.min_y / page_info.height) * rect.height();
            let annot_rect =
                Rect::from_min_max(pos2(x0.min(x1), y0.min(y1)), pos2(x0.max(x1), y0.max(y1)));

            let color = Color32::from_rgb(
                (a.color[0] * 255.0) as u8,
                (a.color[1] * 255.0) as u8,
                (a.color[2] * 255.0) as u8,
            );

            if let Some(ref text) = a.contents {
                let bg_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 25);
                ui.painter()
                    .rect_filled(annot_rect, CornerRadius::same(3), bg_color);
                ui.painter().rect_stroke(
                    annot_rect,
                    CornerRadius::same(3),
                    Stroke::new(1.0, color),
                    StrokeKind::Inside,
                );
                let font_size = (13.0 * state.zoom).clamp(9.0, 24.0);
                ui.painter().text(
                    annot_rect.min + vec2(4.0, 2.0),
                    Align2::LEFT_TOP,
                    text,
                    egui::FontId::proportional(font_size),
                    color,
                );
            } else if a.annot_type == rustpdf_annot::AnnotType::Circle {
                ui.painter().circle_stroke(
                    annot_rect.center(),
                    annot_rect.width() * 0.45,
                    Stroke::new(2.0, color),
                );
                ui.painter()
                    .circle_filled(annot_rect.center(), annot_rect.width() * 0.20, color);
            } else if a.annot_type == rustpdf_annot::AnnotType::Line {
                ui.painter().line_segment(
                    [
                        pos2(annot_rect.min.x, annot_rect.center().y),
                        pos2(annot_rect.max.x, annot_rect.center().y),
                    ],
                    Stroke::new(2.0, color),
                );
            }
        }
    }
}
