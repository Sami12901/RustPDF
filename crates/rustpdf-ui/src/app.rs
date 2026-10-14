//! rustpdf-ui/app — Main desktop application shell, tabs, menus, shortcuts, and workspaces.

use eframe::App;
use egui::{
    vec2, Align, Button, CentralPanel, Color32, Context, Key, Layout, MenuBar, Modifiers, Panel,
    RichText, Ui,
};
use rustpdf_engine::DocumentSession;
use rustpdf_render::RenderPool;
use std::path::Path;

use crate::home::{HomeAction, HomeView, RecentFilesManager};
use crate::modals::{ModalAction, ModalsComponent, ModalsState};
use crate::organizer::{OrganizerAction, OrganizerComponent, OrganizerState};
use crate::search::{SearchBarComponent, SearchState};
use crate::theme::AppTheme;
use crate::thumbnails::ThumbnailConfig;
use crate::viewer::{EditTool, StampKind, ViewMode, ViewerComponent, ViewerState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveWorkspace {
    Home,
    Viewer,
    Organizer,
}

pub struct RustPdfApp {
    pub theme: AppTheme,
    pub thumbnails: ThumbnailConfig,
    pub recent: RecentFilesManager,

    // Document Sessions (Tabs)
    pub sessions: Vec<DocumentSession>,
    pub active_tab: Option<usize>,

    // Sub-component states
    pub active_workspace: ActiveWorkspace,
    pub viewer_state: ViewerState,
    pub organizer_state: OrganizerState,
    pub search_state: SearchState,
    pub modals_state: ModalsState,

    // Asynchronous rendering
    pub render_pool: RenderPool,

    // Sidebar & Tools Drawer
    pub show_sidebar: bool,
    pub show_tools_panel: bool,
    pub drawer_page: crate::tools_panel::DrawerPage,

    // Status / Operation Notifications: (message, is_error, timestamp)
    pub status_notification: Option<(String, bool, std::time::Instant)>,

    // Pending close requests are held until the user saves, discards, or cancels.
    pub pending_close_tab: Option<usize>,
    pub show_exit_confirmation: bool,
}

impl RustPdfApp {
    pub fn new_with_initial_file(
        cc: &eframe::CreationContext<'_>,
        initial_file: Option<std::path::PathBuf>,
    ) -> Self {
        let mut app = Self::new(cc);
        if let Some(path) = initial_file {
            if path.exists() {
                app.open_file(path);
            }
        }
        app
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = AppTheme::load_persisted();
        theme.apply_to_egui(&cc.egui_ctx);
        crate::fonts::setup_global_fonts(&cc.egui_ctx);

        let thumbnails = ThumbnailConfig::load_persisted();
        let recent = RecentFilesManager::load_persisted();
        let render_pool = RenderPool::new(4);

        Self {
            theme,
            thumbnails,
            recent,
            sessions: Vec::new(),
            active_tab: None,
            active_workspace: ActiveWorkspace::Home,
            viewer_state: ViewerState::default(),
            organizer_state: OrganizerState::default(),
            search_state: SearchState::default(),
            modals_state: ModalsState::default(),
            render_pool,
            show_sidebar: true,
            show_tools_panel: false,
            drawer_page: crate::tools_panel::DrawerPage::AllTools,
            status_notification: None,
            pending_close_tab: None,
            show_exit_confirmation: false,
        }
    }

    pub fn notify_success(&mut self, msg: impl Into<String>) {
        self.status_notification = Some((msg.into(), false, std::time::Instant::now()));
    }

    pub fn notify_error(&mut self, msg: impl Into<String>) {
        self.status_notification = Some((msg.into(), true, std::time::Instant::now()));
    }

    fn invalidate_active_document_views(&mut self) {
        if let Some(session) = self.active_session() {
            let doc_id = session.id;
            self.viewer_state.invalidate_doc(doc_id);
            self.organizer_state.invalidate_doc(doc_id);
        }
    }

    fn report_document_operation<E: std::fmt::Display>(
        &mut self,
        result: Option<Result<(), E>>,
        success_message: impl Into<String>,
    ) {
        match result {
            Some(Ok(())) => {
                self.notify_success(success_message);
                self.invalidate_active_document_views();
            }
            Some(Err(error)) => self.notify_error(format!("{error}")),
            None => self.notify_error("Open a PDF before using this tool."),
        }
    }

    fn report_history_operation<E: std::fmt::Display>(
        &mut self,
        result: Option<Result<bool, E>>,
        action: &str,
    ) {
        match result {
            Some(Ok(true)) => {
                self.notify_success(format!("{action} completed."));
                self.invalidate_active_document_views();
            }
            Some(Ok(false)) => self.notify_error(format!("Nothing to {action.to_lowercase()}.")),
            Some(Err(error)) => self.notify_error(format!("{action} failed: {error}")),
            None => self.notify_error("Open a PDF before using undo/redo."),
        }
    }

    pub fn save_active_document(&mut self) {
        let Some(idx) = self.active_tab else {
            return;
        };
        let session = &mut self.sessions[idx];
        let target = if let Some(ref p) = session.file_path.clone() {
            Some(p.clone())
        } else {
            rfd::FileDialog::new()
                .add_filter("PDF", &["pdf"])
                .save_file()
        };

        if let Some(path) = target {
            match session.save_to(&path) {
                Ok(()) => {
                    self.notify_success(format!("Saved: {}", path.display()));
                }
                Err(e) => {
                    self.notify_error(format!("Failed to save: {e}"));
                }
            }
        }
    }

    pub fn save_active_document_as(&mut self) {
        let Some(idx) = self.active_tab else {
            return;
        };
        let session = &mut self.sessions[idx];
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .save_file()
        {
            match session.save_to(&path) {
                Ok(()) => {
                    self.notify_success(format!("Saved as: {}", path.display()));
                }
                Err(e) => {
                    self.notify_error(format!("Failed to save as: {e}"));
                }
            }
        }
    }

    pub fn open_file(&mut self, path: impl AsRef<Path>) {
        let p = path.as_ref();
        // Check if file is already open in an existing session
        if let Some(existing_idx) = self
            .sessions
            .iter()
            .position(|s| s.file_path.as_deref() == Some(p))
        {
            self.active_tab = Some(existing_idx);
            self.active_workspace = ActiveWorkspace::Viewer;
            let doc_id = self.sessions[existing_idx].id;
            if self.viewer_state.active_doc_id != Some(doc_id) {
                self.viewer_state.active_doc_id = Some(doc_id);
                self.viewer_state.current_page = 0;
                self.viewer_state.selected_edit = None;
                self.viewer_state.new_text_placement = None;
            }
            return;
        }

        match DocumentSession::open_file(p) {
            Ok(session) => {
                let page_count = session.page_count();
                let file_size = session.raw_bytes.len();
                let doc_id = session.id;
                self.recent.add_or_update(p, page_count, file_size);

                self.sessions.push(session);
                self.active_tab = Some(self.sessions.len() - 1);
                self.active_workspace = ActiveWorkspace::Viewer;
                self.viewer_state.active_doc_id = Some(doc_id);
                self.viewer_state.current_page = 0;
                self.viewer_state.selected_edit = None;
                self.viewer_state.new_text_placement = None;
            }
            Err(e) => {
                self.notify_error(format!("Failed to open PDF: {e}"));
            }
        }
    }

    pub fn new_blank_document(&mut self) {
        match rustpdf_export::create_blank_pdf(1, 612.0, 792.0, "Untitled") {
            Ok(bytes) => match DocumentSession::open_mem(bytes, None) {
                Ok(session) => {
                    let doc_id = session.id;
                    self.sessions.push(session);
                    self.active_tab = Some(self.sessions.len() - 1);
                    self.active_workspace = ActiveWorkspace::Viewer;
                    self.viewer_state.active_doc_id = Some(doc_id);
                    self.viewer_state.current_page = 0;
                    self.viewer_state.selected_edit = None;
                    self.viewer_state.new_text_placement = None;
                    self.notify_success("Created a new blank PDF.");
                }
                Err(error) => self.notify_error(format!("Failed to create a blank PDF: {error}")),
            },
            Err(error) => self.notify_error(format!("Failed to generate a blank PDF: {error}")),
        }
    }

    /// Requests a tab close without discarding unsaved edits implicitly.
    pub fn close_tab(&mut self, index: usize) {
        if let Some(session) = self.sessions.get(index) {
            if session.is_dirty {
                self.pending_close_tab = Some(index);
                return;
            }
        }
        self.close_tab_now(index);
    }

    fn close_tab_now(&mut self, index: usize) {
        if index < self.sessions.len() {
            let removed = self.sessions.remove(index);
            self.viewer_state.invalidate_doc(removed.id);
            self.organizer_state.invalidate_doc(removed.id);
            self.pending_close_tab = None;
            if self.sessions.is_empty() {
                self.active_tab = None;
                self.active_workspace = ActiveWorkspace::Home;
                self.viewer_state.active_doc_id = None;
            } else {
                let cur = self.active_tab.unwrap_or(0);
                let new_idx = if cur >= self.sessions.len() {
                    self.sessions.len() - 1
                } else if cur > index {
                    cur - 1
                } else {
                    cur
                };
                self.active_tab = Some(new_idx);
                let new_id = self.sessions[new_idx].id;
                self.viewer_state.active_doc_id = Some(new_id);
            }
        }
    }

    fn save_all_dirty(&mut self) -> Result<(), String> {
        for index in 0..self.sessions.len() {
            if !self.sessions[index].is_dirty {
                continue;
            }

            let existing_path = self.sessions[index].file_path.clone();
            let target_path = existing_path.or_else(|| {
                rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name(&self.sessions[index].display_name)
                    .save_file()
            });
            let Some(path) = target_path else {
                return Err("Save cancelled; RustPDF remains open.".to_string());
            };

            self.sessions[index]
                .save_to(&path)
                .map_err(|error| format!("Failed to save '{}': {error}", path.display()))?;
        }
        Ok(())
    }

    fn render_close_confirmations(&mut self, ctx: &Context) {
        if let Some(index) = self.pending_close_tab {
            if index >= self.sessions.len() {
                self.pending_close_tab = None;
            } else {
                let name = self.sessions[index].display_name.clone();
                let mut dialog_open = true;
                let mut save_and_close = false;
                let mut discard = false;
                let mut cancel = false;

                egui::Window::new("Unsaved Changes")
                    .id(egui::Id::new("close_tab_unsaved_changes"))
                    .open(&mut dialog_open)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!("'{}' has unsaved changes.", name));
                        ui.label("Save your edits before closing this tab?");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Save & Close").clicked() {
                                save_and_close = true;
                            }
                            if ui.button("Discard Changes").clicked() {
                                discard = true;
                            }
                            if ui.button("Cancel").clicked() {
                                cancel = true;
                            }
                        });
                    });

                if cancel || !dialog_open {
                    self.pending_close_tab = None;
                } else if discard {
                    self.close_tab_now(index);
                } else if save_and_close {
                    self.active_tab = Some(index);
                    self.active_workspace = ActiveWorkspace::Viewer;
                    self.save_active_document();
                    if self
                        .sessions
                        .get(index)
                        .is_some_and(|session| !session.is_dirty)
                    {
                        self.close_tab_now(index);
                    }
                }
            }
        }

        if self.show_exit_confirmation {
            let mut dialog_open = true;
            let mut save_and_exit = false;
            let mut discard_and_exit = false;
            let mut cancel = false;

            egui::Window::new("Exit RustPDF?")
                .id(egui::Id::new("exit_unsaved_changes"))
                .open(&mut dialog_open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("One or more documents have unsaved changes.");
                    ui.label("Save all changes before exiting?");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save All & Exit").clicked() {
                            save_and_exit = true;
                        }
                        if ui.button("Discard & Exit").clicked() {
                            discard_and_exit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });
                });

            if cancel || !dialog_open {
                self.show_exit_confirmation = false;
            } else if discard_and_exit {
                self.show_exit_confirmation = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if save_and_exit {
                match self.save_all_dirty() {
                    Ok(()) => {
                        self.show_exit_confirmation = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    Err(error) => self.notify_error(error),
                }
            }
        }
    }

    pub fn active_session(&self) -> Option<&DocumentSession> {
        let idx = self.active_tab?;
        self.sessions.get(idx)
    }

    pub fn active_session_mut(&mut self) -> Option<&mut DocumentSession> {
        let idx = self.active_tab?;
        self.sessions.get_mut(idx)
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        // Avoid intercepting shortcuts intended for active text-editing controls.
        if ctx.wants_keyboard_input() {
            return;
        }

        // Ctrl+O: Open
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::O) || i.consume_key(Modifiers::CTRL, Key::O)
        }) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PDF", &["pdf"])
                .pick_file()
            {
                self.open_file(path);
            }
        }

        // Ctrl+S: Save
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::S) || i.consume_key(Modifiers::CTRL, Key::S)
        }) {
            self.save_active_document();
        }

        // Ctrl+F: Find
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::F) || i.consume_key(Modifiers::CTRL, Key::F)
        }) {
            self.search_state.open();
        }

        // Ctrl+H: Find and Replace
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::H) || i.consume_key(Modifiers::CTRL, Key::H)
        }) {
            self.search_state.open();
            self.search_state.is_replace_open = true;
        }

        // Ctrl+K: Command Palette
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::K) || i.consume_key(Modifiers::CTRL, Key::K)
        }) {
            self.modals_state.show_command_palette = !self.modals_state.show_command_palette;
        }

        // Ctrl+Z: Undo
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::Z) || i.consume_key(Modifiers::CTRL, Key::Z)
        }) {
            let result = self.active_session_mut().map(|session| session.undo());
            self.report_history_operation(result, "Undo");
        }

        // Ctrl+Y: Redo
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::Y) || i.consume_key(Modifiers::CTRL, Key::Y)
        }) {
            let result = self.active_session_mut().map(|session| session.redo());
            self.report_history_operation(result, "Redo");
        }

        // Ctrl+= / Ctrl++: Zoom in
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::Equals)
                || i.consume_key(Modifiers::CTRL, Key::Equals)
        }) {
            self.viewer_state.zoom_in();
        }

        // Ctrl+-: Zoom out
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::Minus)
                || i.consume_key(Modifiers::CTRL, Key::Minus)
        }) {
            self.viewer_state.zoom_out();
        }

        // Ctrl+0 / Ctrl+1: Fit Page / 100% Zoom
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::Num0)
                || i.consume_key(Modifiers::CTRL, Key::Num0)
                || i.consume_key(Modifiers::COMMAND, Key::Num1)
                || i.consume_key(Modifiers::CTRL, Key::Num1)
        }) {
            self.viewer_state.set_zoom(1.0);
        }

        // Ctrl+2: Fit Width
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::Num2)
                || i.consume_key(Modifiers::CTRL, Key::Num2)
        }) {
            if let Some(s) = self.active_session() {
                if let Some(p) = s.model.pages.get(self.viewer_state.current_page) {
                    self.viewer_state.fit_width(850.0, p.width);
                }
            }
        }

        // Ctrl+W: Close active tab
        if ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::W) || i.consume_key(Modifiers::CTRL, Key::W)
        }) {
            if let Some(cur) = self.active_tab {
                self.close_tab(cur);
            }
        }

        // Ctrl+Tab / Ctrl+Shift+Tab: Switch active tabs
        let (cycle_forward, cycle_backward) = ctx.input_mut(|i| {
            let ctrl = i.modifiers.ctrl || i.modifiers.command;
            let fwd = ctrl && !i.modifiers.shift && i.consume_key(Modifiers::NONE, Key::Tab);
            let bwd = ctrl && i.modifiers.shift && i.consume_key(Modifiers::NONE, Key::Tab);
            (fwd, bwd)
        });
        if !self.sessions.is_empty() {
            if cycle_forward {
                let next = self
                    .active_tab
                    .map_or(0, |curr| (curr + 1) % self.sessions.len());
                self.active_tab = Some(next);
                self.active_workspace = ActiveWorkspace::Viewer;
            } else if cycle_backward {
                let prev = self.active_tab.map_or(0, |curr| {
                    if curr == 0 {
                        self.sessions.len() - 1
                    } else {
                        curr - 1
                    }
                });
                self.active_tab = Some(prev);
                self.active_workspace = ActiveWorkspace::Viewer;
            }
        }

        // PageDown / PageUp: Navigate pages in viewer
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::PageDown)) {
            if let Some(s) = self.active_session() {
                if self.viewer_state.current_page + 1 < s.page_count() {
                    self.viewer_state.current_page += 1;
                }
            }
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::PageUp)) {
            if self.viewer_state.current_page > 0 {
                self.viewer_state.current_page -= 1;
            }
        }
    }
}

impl App for RustPdfApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Intercept the native window close when any document is dirty.
        if ctx.input(|input| input.viewport().close_requested())
            && self.sessions.iter().any(|session| session.is_dirty)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_exit_confirmation = true;
        }

        // Re-resolve System mode every frame so OS theme changes are reflected live.
        self.theme.apply_to_egui(&ctx);
        self.handle_shortcuts(&ctx);
        let accent = self.theme.active_accent();

        // 1. Top Menu Bar
        Panel::top("top_menu_bar").show(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.horizontal(|ui| {
                    let (logo_rect, logo_resp) =
                        ui.allocate_exact_size(vec2(18.0, 20.0), egui::Sense::click());
                    crate::icons::Icons::draw_logo(ui.painter(), logo_rect, accent);
                    if ui
                        .selectable_label(
                            self.show_tools_panel,
                            RichText::new("RustPdf").strong().color(accent),
                        )
                        .clicked()
                        || logo_resp.clicked()
                    {
                        self.show_tools_panel = !self.show_tools_panel;
                    }
                });
                ui.separator();

                if ui
                    .selectable_label(self.show_tools_panel, "All Tools")
                    .clicked()
                {
                    self.show_tools_panel = !self.show_tools_panel;
                }
                ui.separator();

                ui.menu_button("File", |ui| {
                    if ui.button("Open PDF... (Ctrl+O)").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("PDF", &["pdf"])
                            .pick_file()
                        {
                            self.open_file(path);
                        }
                    }

                    if ui.button("New Blank PDF").clicked() {
                        self.new_blank_document();
                    }

                    ui.separator();

                    let has_active = self.active_tab.is_some();

                    if ui
                        .add_enabled(has_active, Button::new("Save (Ctrl+S)"))
                        .clicked()
                    {
                        self.save_active_document();
                    }

                    if ui
                        .add_enabled(has_active, Button::new("Save As..."))
                        .clicked()
                    {
                        self.save_active_document_as();
                    }

                    ui.separator();

                    if ui
                        .add_enabled(has_active, Button::new("Close Tab"))
                        .clicked()
                    {
                        if let Some(cur) = self.active_tab {
                            self.close_tab(cur);
                        }
                    }

                    if ui.button("Exit").clicked() {
                        if self.sessions.iter().any(|session| session.is_dirty) {
                            self.show_exit_confirmation = true;
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let can_undo = self
                        .active_session_mut()
                        .map(|s| s.history.can_undo())
                        .unwrap_or(false);
                    let can_redo = self
                        .active_session_mut()
                        .map(|s| s.history.can_redo())
                        .unwrap_or(false);

                    if ui
                        .add_enabled(can_undo, Button::new("Undo (Ctrl+Z)"))
                        .clicked()
                    {
                        let result = self.active_session_mut().map(|session| session.undo());
                        self.report_history_operation(result, "Undo");
                    }

                    if ui
                        .add_enabled(can_redo, Button::new("Redo (Ctrl+Y)"))
                        .clicked()
                    {
                        let result = self.active_session_mut().map(|session| session.redo());
                        self.report_history_operation(result, "Redo");
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Zoom In (Ctrl+)").clicked() {
                        self.viewer_state.zoom_in();
                    }
                    if ui.button("Zoom Out (Ctrl-)").clicked() {
                        self.viewer_state.zoom_out();
                    }
                    if ui.button("Actual Size (100%)").clicked() {
                        self.viewer_state.set_zoom(1.0);
                    }
                    ui.separator();
                    if ui
                        .selectable_label(
                            self.viewer_state.view_mode == ViewMode::Continuous,
                            "Continuous Scroll",
                        )
                        .clicked()
                    {
                        self.viewer_state.view_mode = ViewMode::Continuous;
                    }
                    if ui
                        .selectable_label(
                            self.viewer_state.view_mode == ViewMode::SinglePage,
                            "Single Page",
                        )
                        .clicked()
                    {
                        self.viewer_state.view_mode = ViewMode::SinglePage;
                    }
                    if ui
                        .selectable_label(
                            self.viewer_state.view_mode == ViewMode::TwoUp,
                            "Two-Page Spread",
                        )
                        .clicked()
                    {
                        self.viewer_state.view_mode = ViewMode::TwoUp;
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui
                        .selectable_label(self.show_tools_panel, "Open All Tools Drawer")
                        .clicked()
                    {
                        self.show_tools_panel = !self.show_tools_panel;
                    }
                    ui.separator();
                    if ui
                        .selectable_label(
                            self.active_workspace == ActiveWorkspace::Organizer,
                            "Organize Pages",
                        )
                        .clicked()
                    {
                        self.active_workspace =
                            if self.active_workspace == ActiveWorkspace::Organizer {
                                ActiveWorkspace::Viewer
                            } else {
                                ActiveWorkspace::Organizer
                            };
                    }
                    if ui.button("Comments & Annotations...").clicked() {
                        self.modals_state.show_annotations_manager = true;
                    }
                    if ui.button("Interactive Forms...").clicked() {
                        self.modals_state.show_forms_manager = true;
                    }
                    if ui.button("Protect Document...").clicked() {
                        self.modals_state.show_protect = true;
                    }
                    if ui.button("Layers & Attachments...").clicked() {
                        self.modals_state.show_layers_manager = true;
                    }
                    ui.separator();
                    if ui.button("Find (Ctrl+F)").clicked() {
                        self.search_state.open();
                    }
                    if ui.button("Combine Files...").clicked() {
                        self.modals_state.show_combine = true;
                    }
                    if ui.button("Split Document...").clicked() {
                        self.modals_state.show_split = true;
                    }
                    if ui.button("Document Properties").clicked() {
                        self.modals_state.show_properties = true;
                    }
                });

                ui.menu_button("Settings", |ui| {
                    if ui.button("Appearance & Themes...").clicked() {
                        self.modals_state.show_appearance = true;
                    }
                    if ui.button("Thumbnail Settings...").clicked() {
                        self.modals_state.show_thumbnails = true;
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About RustPdf...").clicked() {
                        self.modals_state.show_about = true;
                    }
                    if ui.button("Command Palette (Ctrl+K)").clicked() {
                        self.modals_state.show_command_palette = true;
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("100% Offline")
                            .size(11.0)
                            .color(Color32::from_gray(140)),
                    );
                    if ui.button("Palette (Ctrl+K)").clicked() {
                        self.modals_state.show_command_palette = true;
                    }
                });
            });
        });

        // 2. Tab Strip Bar
        Panel::top("tab_strip_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 0.0);

                // All Tools Drawer Toggle
                if ui
                    .selectable_label(self.show_tools_panel, "All Tools")
                    .on_hover_text("Open All Tools Drawer")
                    .clicked()
                {
                    self.show_tools_panel = !self.show_tools_panel;
                }

                // Home Tab Button
                let is_home = self.active_workspace == ActiveWorkspace::Home;
                if ui.selectable_label(is_home, "Home").clicked() {
                    self.active_workspace = ActiveWorkspace::Home;
                }

                ui.separator();

                let mut to_close = None;

                for (idx, session) in self.sessions.iter().enumerate() {
                    let is_active = Some(idx) == self.active_tab
                        && self.active_workspace != ActiveWorkspace::Home;

                    let tab_title = if session.is_dirty {
                        format!("• {}", session.display_name)
                    } else {
                        session.display_name.clone()
                    };

                    let resp = ui.selectable_label(is_active, tab_title);
                    if resp.clicked() {
                        if self.active_tab != Some(idx) {
                            self.viewer_state.active_doc_id = Some(session.id);
                            self.viewer_state.current_page = 0;
                            self.viewer_state.selected_edit = None;
                            self.viewer_state.new_text_placement = None;
                        }
                        self.active_tab = Some(idx);
                        self.active_workspace = ActiveWorkspace::Viewer;
                    }

                    if ui.button("×").clicked() {
                        to_close = Some(idx);
                    }
                }

                if let Some(idx) = to_close {
                    self.close_tab(idx);
                }

                if ui.button("+").on_hover_text("New blank document").clicked() {
                    self.new_blank_document();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.active_tab.is_some() {
                        let is_org = self.active_workspace == ActiveWorkspace::Organizer;
                        if ui.selectable_label(is_org, "Organize Pages").clicked() {
                            self.active_workspace = if is_org {
                                ActiveWorkspace::Viewer
                            } else {
                                ActiveWorkspace::Organizer
                            };
                        }
                    }
                });
            });
        });

        // 3. Search Bar (if opened)
        if self.search_state.is_open {
            if let Some(idx) = self.active_tab {
                if let Some(session) = self.sessions.get(idx) {
                    self.search_state.update_search(session, session.id, false);
                }
            }
            Panel::top("search_bar_panel").show(ui, |ui| {
                if let Some(action) = SearchBarComponent::render(ui, &mut self.search_state, accent)
                {
                    match action {
                        crate::search::SearchBarAction::Navigate(target_page) => {
                            self.viewer_state.current_page = target_page;
                        }
                        crate::search::SearchBarAction::ReplaceOne => {
                            if let Some(hit) = self.search_state.current_match().cloned() {
                                let q = self.search_state.query.clone();
                                let repl = self.search_state.replace_query.clone();
                                let is_case = self.search_state.case_sensitive;
                                if let Some(idx) = self.active_tab {
                                    let session = &mut self.sessions[idx];
                                    let doc_id = session.id;
                                    let blocks = session.get_page_text_blocks(hit.page_index);
                                    let mut replacement_done = false;
                                    if let Some(target_block) = blocks.into_iter().find(|b| {
                                        if is_case {
                                            b.text.contains(&q)
                                        } else {
                                            b.text.to_lowercase().contains(&q.to_lowercase())
                                        }
                                    }) {
                                        let new_text = if is_case {
                                            target_block.text.replacen(&q, &repl, 1)
                                        } else {
                                            let lower_text = target_block.text.to_lowercase();
                                            let lower_q = q.to_lowercase();
                                            if let Some(pos) = lower_text.find(&lower_q) {
                                                let mut s = target_block.text.clone();
                                                s.replace_range(pos..pos + q.len(), &repl);
                                                s
                                            } else {
                                                target_block.text.clone()
                                            }
                                        };
                                        if new_text != target_block.text {
                                            let _ = session.replace_text_block(
                                                hit.page_index,
                                                &target_block,
                                                &new_text,
                                            );
                                            replacement_done = true;
                                        }
                                    }
                                    if replacement_done {
                                        self.viewer_state
                                            .invalidate_page_texture(doc_id, hit.page_index);
                                        self.viewer_state
                                            .text_blocks_cache
                                            .retain(|&(d, _), _| d != doc_id);
                                        let session_ref = &self.sessions[idx];
                                        self.search_state.update_search(session_ref, doc_id, true);
                                        self.notify_success("Replaced 1 match");
                                    }
                                }
                            }
                        }
                        crate::search::SearchBarAction::ReplaceAll => {
                            let q = self.search_state.query.clone();
                            let repl = self.search_state.replace_query.clone();
                            let is_case = self.search_state.case_sensitive;
                            if !q.is_empty() {
                                if let Some(idx) = self.active_tab {
                                    let session = &mut self.sessions[idx];
                                    let doc_id = session.id;
                                    let mut count = 0;
                                    let mut modified_pages = Vec::new();
                                    let num_pages = session.page_count();
                                    for p_idx in 0..num_pages {
                                        let blocks = session.get_page_text_blocks(p_idx);
                                        let mut page_changed = false;
                                        for block in blocks {
                                            let has_match = if is_case {
                                                block.text.contains(&q)
                                            } else {
                                                block
                                                    .text
                                                    .to_lowercase()
                                                    .contains(&q.to_lowercase())
                                            };
                                            if has_match {
                                                let new_text = if is_case {
                                                    block.text.replace(&q, &repl)
                                                } else {
                                                    let lower_q = q.to_lowercase();
                                                    let mut result = String::new();
                                                    let mut last_end = 0;
                                                    let lower_text = block.text.to_lowercase();
                                                    while let Some(pos) =
                                                        lower_text[last_end..].find(&lower_q)
                                                    {
                                                        let abs_pos = last_end + pos;
                                                        result.push_str(
                                                            &block.text[last_end..abs_pos],
                                                        );
                                                        result.push_str(&repl);
                                                        last_end = abs_pos + q.len();
                                                    }
                                                    result.push_str(&block.text[last_end..]);
                                                    result
                                                };
                                                if new_text != block.text {
                                                    let _ = session.replace_text_block(
                                                        p_idx, &block, &new_text,
                                                    );
                                                    page_changed = true;
                                                    count += 1;
                                                }
                                            }
                                        }
                                        if page_changed {
                                            modified_pages.push(p_idx);
                                        }
                                    }
                                    for p_idx in modified_pages {
                                        self.viewer_state.invalidate_page_texture(doc_id, p_idx);
                                    }
                                    if count > 0 {
                                        self.viewer_state
                                            .text_blocks_cache
                                            .retain(|&(d, _), _| d != doc_id);
                                        let session_ref = &self.sessions[idx];
                                        self.search_state.update_search(session_ref, doc_id, true);
                                        self.notify_success(format!(
                                            "Replaced {} occurrences across document",
                                            count
                                        ));
                                    } else {
                                        self.notify_error("No matches found to replace");
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

        // 4. Bottom Status Bar
        Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(idx) = self.active_tab {
                    if let Some(session) = self.sessions.get(idx) {
                        let total = session.page_count();
                        ui.label(format!(
                            "Pages: {} | Zoom: {:.0}%",
                            total,
                            self.viewer_state.zoom * 100.0
                        ));
                        if session.is_dirty {
                            ui.label(RichText::new("Unsaved Changes").color(accent).size(12.0));
                        }
                    }
                } else {
                    ui.label("RustPdf Workbench — Ready");
                }

                if let Some((ref msg, is_err, timestamp)) = self.status_notification {
                    if timestamp.elapsed().as_secs() < 6 {
                        let color = if is_err {
                            Color32::from_rgb(239, 68, 68)
                        } else {
                            Color32::from_rgb(34, 197, 94)
                        };
                        let prefix = if is_err { "⚠ " } else { "✓ " };
                        ui.separator();
                        ui.label(
                            RichText::new(format!("{prefix}{msg}"))
                                .color(color)
                                .strong()
                                .size(12.0),
                        );
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("RustPDF v{}", env!("CARGO_PKG_VERSION")))
                            .size(11.0)
                            .color(Color32::from_gray(130)),
                    );
                    let (small_logo, _) =
                        ui.allocate_exact_size(vec2(14.0, 16.0), egui::Sense::hover());
                    crate::icons::Icons::draw_logo(ui.painter(), small_logo, accent);
                });
            });
        });

        // 5. Left All Tools Drawer (when toggled open)
        if self.show_tools_panel {
            Panel::left("all_tools_side_panel")
                .default_size(260.0)
                .resizable(true)
                .show(ui, |ui| {
                    let has_session =
                        self.active_tab.is_some() && self.active_workspace != ActiveWorkspace::Home;
                    let action = crate::tools_panel::ToolsPanelComponent::render(
                        ui,
                        &mut self.drawer_page,
                        accent,
                        has_session,
                        self.viewer_state.is_edit_mode,
                    );

                    if let Some(act) = action {
                        match act {
                            crate::tools_panel::ToolAction::CloseDrawer => {
                                self.show_tools_panel = false;
                            }
                            crate::tools_panel::ToolAction::NavigateTo(page) => {
                                self.drawer_page = page;
                            }
                            crate::tools_panel::ToolAction::NavigateBack => {
                                self.drawer_page = crate::tools_panel::DrawerPage::AllTools;
                            }
                            crate::tools_panel::ToolAction::ExportPdf => {
                                self.drawer_page = crate::tools_panel::DrawerPage::ExportPdf;
                            }
                            crate::tools_panel::ToolAction::FillAndSign => {
                                self.drawer_page = crate::tools_panel::DrawerPage::FillAndSign;
                            }
                            crate::tools_panel::ToolAction::EditPdf => {
                                self.drawer_page = crate::tools_panel::DrawerPage::EditPdf;
                            }
                            crate::tools_panel::ToolAction::RotatePages => {
                                let curr = self.viewer_state.current_page;
                                let result = self
                                    .active_session_mut()
                                    .map(|session| session.rotate_pages(&[curr], 90));
                                self.report_document_operation(
                                    result,
                                    format!("Rotated page {}.", curr + 1),
                                );
                            }
                            crate::tools_panel::ToolAction::InsertPages => {
                                let curr = self.viewer_state.current_page;
                                let result = self
                                    .active_session_mut()
                                    .map(|session| session.insert_blank_page(curr + 1, 595.0, 842.0));
                                self.report_document_operation(
                                    result,
                                    format!("Inserted a blank page after page {}.", curr + 1),
                                );
                            }
                            crate::tools_panel::ToolAction::DeletePages => {
                                let curr = self.viewer_state.current_page;
                                let result = self
                                    .active_session_mut()
                                    .map(|session| session.delete_pages(&[curr]));
                                self.report_document_operation(
                                    result,
                                    format!("Deleted page {}.", curr + 1),
                                );
                            }
                            crate::tools_panel::ToolAction::ExtractPages => {
                                let selected_pages: Vec<usize> =
                                    if self.active_workspace == ActiveWorkspace::Organizer
                                        && !self.organizer_state.selected_pages.is_empty()
                                    {
                                        self.organizer_state
                                            .selected_pages
                                            .iter()
                                            .copied()
                                            .collect()
                                    } else {
                                        vec![self.viewer_state.current_page]
                                    };

                                let source = self.active_session().map(|session| {
                                    (
                                        session.raw_bytes.as_ref().clone(),
                                        session.file_path.clone(),
                                        session.display_name.clone(),
                                    )
                                });

                                if let Some((source_bytes, source_path, display_name)) = source {
                                    let default_name = format!(
                                        "{}-extracted.pdf",
                                        display_name.trim_end_matches(".pdf")
                                    );
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("PDF", &["pdf"])
                                        .set_file_name(&default_name)
                                        .save_file()
                                    {
                                        if source_path.as_deref() == Some(path.as_path()) {
                                            self.notify_error(
                                                "Choose a different output file so the source PDF is not overwritten."
                                            );
                                        } else {
                                            match rustpdf_organize::extract_pages(
                                                &source_bytes,
                                                &selected_pages,
                                            ) {
                                                Ok(extracted) => {
                                                    match rustpdf_edit::save_atomic(&path, &extracted) {
                                                        Ok(()) => self.notify_success(format!(
                                                    "Extracted {} page(s) to {}",
                                                    selected_pages
                                                        .iter()
                                                        .copied()
                                                        .collect::<std::collections::BTreeSet<_>>()
                                                        .len(),
                                                    path.display()
                                                )),
                                                        Err(error) => self.notify_error(format!(
                                                            "Failed to save extracted PDF: {error}"
                                                        )),
                                                    }
                                                }
                                                Err(error) => self.notify_error(format!(
                                                    "Failed to extract pages: {error}"
                                                )),
                                            }
                                        }
                                    }
                                } else {
                                    self.notify_error("Open a PDF before extracting pages.");
                                }
                            }
                            crate::tools_panel::ToolAction::EditTextAndImages => {
                                self.viewer_state.is_edit_mode = !self.viewer_state.is_edit_mode;
                                self.viewer_state.active_tool = crate::viewer::EditTool::Select;
                                let doc_id = self.active_session().map(|s| s.id);
                                if let Some(id) = doc_id {
                                    self.viewer_state
                                        .text_blocks_cache
                                        .retain(|&(d, _), _| d != id);
                                }
                                self.active_workspace = ActiveWorkspace::Viewer;
                            }
                            crate::tools_panel::ToolAction::AddTextContent => {
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool = crate::viewer::EditTool::AddText;
                                self.active_workspace = ActiveWorkspace::Viewer;
                            }
                            crate::tools_panel::ToolAction::AddImageContent => {
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool = crate::viewer::EditTool::AddImage;
                                self.active_workspace = ActiveWorkspace::Viewer;
                            }
                            crate::tools_panel::ToolAction::Watermark => {
                                self.modals_state.target_page = self.viewer_state.current_page + 1;
                                self.modals_state.edit_content_tab =
                                    crate::modals::EditContentTab::Watermark;
                                self.modals_state.show_edit_content_dialog = true;
                            }
                            crate::tools_panel::ToolAction::Link => {
                                self.modals_state.target_page = self.viewer_state.current_page + 1;
                                self.modals_state.edit_content_tab =
                                    crate::modals::EditContentTab::Link;
                                self.modals_state.show_edit_content_dialog = true;
                            }
                            crate::tools_panel::ToolAction::HeaderAndFooter => {
                                self.modals_state.target_page = self.viewer_state.current_page + 1;
                                self.modals_state.edit_content_tab =
                                    crate::modals::EditContentTab::HeaderFooter;
                                self.modals_state.show_edit_content_dialog = true;
                            }
                            crate::tools_panel::ToolAction::BatesNumbering => {
                                self.modals_state.target_page = self.viewer_state.current_page + 1;
                                self.modals_state.edit_content_tab =
                                    crate::modals::EditContentTab::Bates;
                                self.modals_state.show_edit_content_dialog = true;
                            }
                            crate::tools_panel::ToolAction::Background => {
                                self.modals_state.target_page = self.viewer_state.current_page + 1;
                                self.modals_state.edit_content_tab =
                                    crate::modals::EditContentTab::Background;
                                self.modals_state.show_edit_content_dialog = true;
                            }
                            crate::tools_panel::ToolAction::ExportWord => {
                                if let Some(s) = self.active_session() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Word-compatible Rich Text", &["rtf"])
                                        .set_file_name(format!(
                                            "{}.rtf",
                                            s.display_name.trim_end_matches(".pdf")
                                        ))
                                        .save_file()
                                    {
                                        match rustpdf_export::try_export_word_doc(
                                            s.cos_doc.inner(),
                                            &s.display_name,
                                        ) {
                                            Ok(document) => match std::fs::write(&path, document) {
                                                Ok(()) => self.notify_success(format!(
                                                    "Exported Word-compatible RTF: {}",
                                                    path.display()
                                                )),
                                                Err(error) => self.notify_error(format!(
                                                    "Export failed: {error}"
                                                )),
                                            },
                                            Err(error) => self.notify_error(format!(
                                                "Text extraction failed; RTF was not exported: {error}"
                                            )),
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::ExportRtf => {
                                if let Some(s) = self.active_session() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Rich Text Format", &["rtf"])
                                        .set_file_name(format!(
                                            "{}.rtf",
                                            s.display_name.trim_end_matches(".pdf")
                                        ))
                                        .save_file()
                                    {
                                        match rustpdf_export::try_export_rtf_doc(
                                            s.cos_doc.inner(),
                                            &s.display_name,
                                        ) {
                                            Ok(rtf) => match std::fs::write(&path, rtf) {
                                                Ok(()) => self.notify_success(format!(
                                                    "Exported RTF: {}",
                                                    path.display()
                                                )),
                                                Err(error) => self.notify_error(format!(
                                                    "Export failed: {error}"
                                                )),
                                            },
                                            Err(error) => self.notify_error(format!(
                                                "Text extraction failed; RTF was not exported: {error}"
                                            )),
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::ExportHtml => {
                                if let Some(s) = self.active_session() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("HTML Web Page", &["html", "htm"])
                                        .set_file_name(format!(
                                            "{}.html",
                                            s.display_name.trim_end_matches(".pdf")
                                        ))
                                        .save_file()
                                    {
                                        match rustpdf_export::try_export_html_doc(
                                            s.cos_doc.inner(),
                                            &s.display_name,
                                        ) {
                                            Ok(html) => match std::fs::write(&path, html) {
                                                Ok(()) => self.notify_success(format!(
                                                    "Exported HTML: {}",
                                                    path.display()
                                                )),
                                                Err(error) => self.notify_error(format!(
                                                    "Export failed: {error}"
                                                )),
                                            },
                                            Err(error) => self.notify_error(format!(
                                                "Text extraction failed; HTML was not exported: {error}"
                                            )),
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::ExportTextPlain => {
                                if let Some(s) = self.active_session() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Plain Text", &["txt"])
                                        .set_file_name(format!(
                                            "{}.txt",
                                            s.display_name.trim_end_matches(".pdf")
                                        ))
                                        .save_file()
                                    {
                                        match rustpdf_export::try_export_plain_text_doc(s.cos_doc.inner()) {
                                            Ok(text) => match std::fs::write(&path, text) {
                                                Ok(()) => self.notify_success(format!(
                                                    "Exported text: {}",
                                                    path.display()
                                                )),
                                                Err(error) => self.notify_error(format!(
                                                    "Export failed: {error}"
                                                )),
                                            },
                                            Err(error) => self.notify_error(format!(
                                                "Text extraction failed; no text file was written: {error}"
                                            )),
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::ExportSpreadsheet => {
                                if let Some(s) = self.active_session() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("CSV Spreadsheet", &["csv"])
                                        .set_file_name(format!(
                                            "{}.csv",
                                            s.display_name.trim_end_matches(".pdf")
                                        ))
                                        .save_file()
                                    {
                                        match rustpdf_export::try_export_csv_doc(s.cos_doc.inner()) {
                                            Ok(csv) => match std::fs::write(&path, csv) {
                                                Ok(()) => self.notify_success(format!(
                                                    "Exported CSV: {}",
                                                    path.display()
                                                )),
                                                Err(error) => self.notify_error(format!(
                                                    "Export failed: {error}"
                                                )),
                                            },
                                            Err(error) => self.notify_error(format!(
                                                "Text extraction failed; CSV was not exported: {error}"
                                            )),
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::ExportPresentation => {
                                if let Some(s) = self.active_session() {
                                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                        let prefix = s.display_name.trim_end_matches(".pdf");
                                        match rustpdf_export::export_all_pages_png(&s.raw_bytes, &folder, &format!("{}_slide", prefix), 2.0) {
                                            Ok(paths) => self.notify_success(format!("Exported {} slide images to: {}", paths.len(), folder.display())),
                                            Err(e) => self.notify_error(format!("Export failed: {e}")),
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::ExportPostScript => {
                                self.notify_error(
                                    "PostScript export is not implemented yet; no file was written."
                                );
                            }
                            crate::tools_panel::ToolAction::ExportImagePng => {
                                let curr = self.viewer_state.current_page;
                                if let Some(s) = self.active_session() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("PNG Image", &["png"])
                                        .set_file_name(format!("{}_page_{}.png", s.display_name.trim_end_matches(".pdf"), curr + 1))
                                        .save_file()
                                    {
                                        match rustpdf_export::export_page_png(&s.raw_bytes, curr, 2.0) {
                                            Ok(bytes) => match std::fs::write(&path, bytes) {
                                                Ok(()) => self.notify_success(format!("Exported Page {} as PNG: {}", curr + 1, path.display())),
                                                Err(e) => self.notify_error(format!("Save failed: {e}")),
                                            },
                                            Err(e) => self.notify_error(format!("Rasterization failed: {e}")),
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::ExportAllImages => {
                                if let Some(s) = self.active_session() {
                                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                        let prefix = s.display_name.trim_end_matches(".pdf");
                                        match rustpdf_export::export_all_pages_png(&s.raw_bytes, &folder, prefix, 2.0) {
                                            Ok(paths) => self.notify_success(format!("Exported {} page images to: {}", paths.len(), folder.display())),
                                            Err(e) => self.notify_error(format!("Export failed: {e}")),
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::FillAddText => {
                                self.active_workspace = ActiveWorkspace::Viewer;
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool = EditTool::AddText;
                                self.notify_success("Click anywhere on the document page to place a new text box");
                            }
                            crate::tools_panel::ToolAction::FillCheckmark => {
                                self.active_workspace = ActiveWorkspace::Viewer;
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool = EditTool::Stamp(StampKind::Checkmark);
                                self.notify_success("Click anywhere on the document page to stamp ✓");
                            }
                            crate::tools_panel::ToolAction::FillCross => {
                                self.active_workspace = ActiveWorkspace::Viewer;
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool = EditTool::Stamp(StampKind::Cross);
                                self.notify_success("Click anywhere on the document page to stamp ✕");
                            }
                            crate::tools_panel::ToolAction::FillDot => {
                                self.active_workspace = ActiveWorkspace::Viewer;
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool = EditTool::Stamp(StampKind::Dot);
                                self.notify_success("Click anywhere on the document page to stamp ●");
                            }
                            crate::tools_panel::ToolAction::FillLine => {
                                self.active_workspace = ActiveWorkspace::Viewer;
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool = EditTool::Stamp(StampKind::Line);
                                self.notify_success("Click anywhere on the document page to stamp —");
                            }
                            crate::tools_panel::ToolAction::FillDate => {
                                self.active_workspace = ActiveWorkspace::Viewer;
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool = EditTool::Stamp(StampKind::Date);
                                self.notify_success("Click anywhere on the document page to stamp today's date");
                            }
                            crate::tools_panel::ToolAction::SignAddSignature => {
                                self.modals_state.target_page = self.viewer_state.current_page + 1;
                                self.modals_state.is_initials_mode = false;
                                self.modals_state.show_signature_dialog = true;
                            }
                            crate::tools_panel::ToolAction::SignAddInitials => {
                                self.modals_state.target_page = self.viewer_state.current_page + 1;
                                self.modals_state.is_initials_mode = true;
                                self.modals_state.show_signature_dialog = true;
                            }
                            crate::tools_panel::ToolAction::CreatePdf => {
                                self.new_blank_document();
                            }
                            crate::tools_panel::ToolAction::CombineFiles
                            | crate::tools_panel::ToolAction::OpenCombineDialog => {
                                self.modals_state.show_combine = true;
                            }
                            crate::tools_panel::ToolAction::OrganizePages
                            | crate::tools_panel::ToolAction::OpenOrganize => {
                                if self.active_tab.is_some() {
                                    self.active_workspace = ActiveWorkspace::Organizer;
                                } else if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("PDF", &["pdf"])
                                    .pick_file()
                                {
                                    self.open_file(path);
                                    self.active_workspace = ActiveWorkspace::Organizer;
                                }
                            }
                            crate::tools_panel::ToolAction::AddComments => {
                                self.modals_state.show_annotations_manager = true;
                            }
                            crate::tools_panel::ToolAction::ScanOcr => {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Document / Image", &["pdf", "png", "jpg", "jpeg"])
                                    .pick_file()
                                {
                                    self.open_file(path);
                                }
                            }
                            crate::tools_panel::ToolAction::ProtectPdf
                            | crate::tools_panel::ToolAction::OpenProtectDialog => {
                                self.modals_state.show_protect = true;
                            }
                            crate::tools_panel::ToolAction::RedactPdf => {
                                // Priority 5: Distinguish Redaction from Cover-up
                                log::warn!("Redaction is unsupported in v0.4.0. True sanitization is planned for a future release.");
                            }
                            crate::tools_panel::ToolAction::CompressPdf => {
                                if let Some(s) = self.active_session_mut() {
                                    match s.compress_document() {
                                        Ok(res) => {
                                            log::info!(
                                                "Compressed PDF: saved {} bytes ({:.1}%), {} streams compressed, {} objects pruned",
                                                res.saved_bytes,
                                                res.saved_ratio() * 100.0,
                                                res.streams_compressed,
                                                res.objects_pruned
                                            );
                                            if let Some(path) = rfd::FileDialog::new()
                                                .add_filter("Optimized PDF", &["pdf"])
                                                .save_file()
                                            {
                                                if let Err(e) = s.save_to(&path) {
                                                    log::error!("Failed to save compressed PDF: {e}");
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("PDF compression failed: {e}");
                                        }
                                    }
                                }
                            }
                            crate::tools_panel::ToolAction::PrepareForm
                            | crate::tools_panel::ToolAction::OpenFormsManager => {
                                self.modals_state.show_forms_manager = true;
                            }
                            crate::tools_panel::ToolAction::AddStamp => {
                                self.active_workspace = ActiveWorkspace::Viewer;
                                self.viewer_state.is_edit_mode = true;
                                self.viewer_state.active_tool =
                                    crate::viewer::EditTool::Stamp(crate::viewer::StampKind::Checkmark);
                                self.notify_success("Stamp tool active: click anywhere on page to stamp");
                            }
                            crate::tools_panel::ToolAction::UseCertificate => {
                                log::warn!("Digital certificate signing is planned for a future release.");
                            }
                            crate::tools_panel::ToolAction::PrintProduction => {
                                self.modals_state.show_properties = true;
                            }
                            crate::tools_panel::ToolAction::MeasureObjects => {
                                self.modals_state.show_properties = true;
                            }
                            crate::tools_panel::ToolAction::CompareFiles => {
                                log::warn!("File comparison is planned for a future release.");
                            }
                            crate::tools_panel::ToolAction::GuidedActions => {
                                self.active_workspace = ActiveWorkspace::Home;
                            }
                            crate::tools_panel::ToolAction::Accessibility => {
                                self.modals_state.show_properties = true;
                            }
                            crate::tools_panel::ToolAction::PdfStandards => {
                                self.modals_state.show_properties = true;
                            }
                            crate::tools_panel::ToolAction::SearchIndex => {
                                self.search_state.open();
                            }
                            crate::tools_panel::ToolAction::JavaScript => {
                                self.modals_state.show_properties = true;
                            }
                            crate::tools_panel::ToolAction::AddStickyNote => {
                                self.modals_state.show_annotations_manager = true;
                            }
                            crate::tools_panel::ToolAction::AddFreehandInk => {
                                self.active_workspace = ActiveWorkspace::Viewer;
                                self.viewer_state.active_tool = crate::viewer::EditTool::Draw;
                                self.notify_success("Pencil tool active: click & drag on any page to draw");
                            }
                            crate::tools_panel::ToolAction::AddHighlight
                            | crate::tools_panel::ToolAction::AddShape => {
                                self.modals_state.show_annotations_manager = true;
                            }
                            crate::tools_panel::ToolAction::ResetFormFields => {
                                if let Some(s) = self.active_session_mut() {
                                    let _ = s.reset_form_fields();
                                }
                            }
                            crate::tools_panel::ToolAction::ExportFormData => {
                                if let Some(s) = self.active_session_mut() {
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
                            }
                            crate::tools_panel::ToolAction::RemoveProtection => {
                                if let Some(s) = self.active_session_mut() {
                                    let _ = s.remove_protection();
                                }
                            }
                            crate::tools_panel::ToolAction::OpenLayersDialog => {
                                self.modals_state.show_layers_manager = true;
                            }
                            crate::tools_panel::ToolAction::AddAttachment => {
                                if let Some(s) = self.active_session_mut() {
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
                            }
                            crate::tools_panel::ToolAction::OpenSplitDialog => {
                                self.modals_state.show_split = true;
                            }
                            crate::tools_panel::ToolAction::OpenPropertiesDialog => {
                                self.modals_state.show_properties = true;
                            }
                            crate::tools_panel::ToolAction::OpenAppearanceDialog => {
                                self.modals_state.show_appearance = true;
                            }
                            crate::tools_panel::ToolAction::OpenAboutDialog => {
                                self.modals_state.show_about = true;
                            }
                        }
                    }
                });
        }

        // 6. Left Outline/Bookmarks Sidebar (when viewing a document)
        if self.show_sidebar
            && self.active_workspace == ActiveWorkspace::Viewer
            && self.active_tab.is_some()
        {
            Panel::left("outline_sidebar")
                .default_size(self.thumbnails.sidebar_width)
                .show(ui, |ui| {
                    ui.heading("Bookmarks & Outline");
                    ui.separator();

                    if let Some(idx) = self.active_tab {
                        let outlines = self.sessions[idx].model.outlines.clone();
                        if outlines.is_empty() {
                            ui.label(
                                RichText::new("No bookmarks in this document.")
                                    .color(Color32::from_gray(140)),
                            );
                        } else {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for item in &outlines {
                                    if ui.button(&item.title).clicked() {
                                        if let Some(p) = item.page_index {
                                            self.viewer_state.current_page = p;
                                        }
                                    }
                                }
                            });
                        }
                    }
                });
        }

        // 7. Central Workspace Panel
        CentralPanel::default().show(ui, |ui| match self.active_workspace {
            ActiveWorkspace::Home => {
                if let Some(action) = HomeView::render(ui, &mut self.recent, accent) {
                    match action {
                        HomeAction::OpenPdf => {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .pick_file()
                            {
                                self.open_file(path);
                            }
                        }
                        HomeAction::OpenFile(p) => {
                            self.open_file(p);
                        }
                        HomeAction::NewBlank => {
                            self.new_blank_document();
                        }
                        HomeAction::CombineDialog => {
                            self.modals_state.show_combine = true;
                        }
                        HomeAction::SplitDialog => {
                            self.modals_state.show_split = true;
                        }
                        HomeAction::AppearanceSettings => {
                            self.modals_state.show_appearance = true;
                        }
                        HomeAction::ThumbnailSettings => {
                            self.modals_state.show_thumbnails = true;
                        }
                        HomeAction::OrganizePages => {
                            if self.active_tab.is_some() {
                                self.active_workspace = ActiveWorkspace::Organizer;
                            } else if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .pick_file()
                            {
                                self.open_file(path);
                                self.active_workspace = ActiveWorkspace::Organizer;
                            }
                        }
                        HomeAction::AddComments => {
                            if self.active_tab.is_some() {
                                self.modals_state.show_annotations_manager = true;
                            } else if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .pick_file()
                            {
                                self.open_file(path);
                                self.modals_state.show_annotations_manager = true;
                            }
                        }
                        HomeAction::PrepareForm => {
                            if self.active_tab.is_some() {
                                self.modals_state.show_forms_manager = true;
                            } else if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .pick_file()
                            {
                                self.open_file(path);
                                self.modals_state.show_forms_manager = true;
                            }
                        }
                        HomeAction::EditPdf => {
                            if self.active_tab.is_some() {
                                self.active_workspace = ActiveWorkspace::Organizer;
                            } else if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .pick_file()
                            {
                                self.open_file(path);
                                self.active_workspace = ActiveWorkspace::Organizer;
                            }
                        }
                        HomeAction::ProtectPdf => {
                            if self.active_tab.is_some() {
                                self.modals_state.show_protect = true;
                            } else if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .pick_file()
                            {
                                self.open_file(path);
                                self.modals_state.show_protect = true;
                            }
                        }
                    }
                }
            }
            ActiveWorkspace::Viewer => {
                if let Some(idx) = self.active_tab {
                    let session = &mut self.sessions[idx];
                    ViewerComponent::render(
                        ui,
                        session,
                        &mut self.viewer_state,
                        &self.render_pool,
                        accent,
                        Some(&self.search_state),
                    );
                }
            }
            ActiveWorkspace::Organizer => {
                let mut action = None;
                if let Some(idx) = self.active_tab {
                    let session = &mut self.sessions[idx];
                    action = OrganizerComponent::render(
                        ui,
                        session,
                        &mut self.organizer_state,
                        &self.thumbnails,
                        &self.render_pool,
                        accent,
                    );
                }

                if let Some(act) = action {
                    let selected_pages: Vec<usize> = self
                        .organizer_state
                        .selected_pages
                        .iter()
                        .copied()
                        .collect();
                    let max_selected = self
                        .organizer_state
                        .selected_pages
                        .iter()
                        .max()
                        .copied()
                        .unwrap_or(0);
                    let first_selected = self.organizer_state.selected_pages.iter().next().copied();

                    if let Some(session) =
                        self.active_tab.and_then(|idx| self.sessions.get_mut(idx))
                    {
                        let doc_id = session.id;
                        match act {
                            OrganizerAction::Rotate(deg) => {
                                let _ = session.rotate_pages(&selected_pages, deg);
                            }
                            OrganizerAction::Delete => {
                                let _ = session.delete_pages(&selected_pages);
                            }
                            OrganizerAction::InsertBlank => {
                                let _ = session.insert_blank_page(max_selected + 1, 612.0, 792.0);
                            }
                            OrganizerAction::Duplicate => {
                                if let Some(first) = first_selected {
                                    let _ = session.duplicate_page(first);
                                }
                            }
                            OrganizerAction::MoveLeft => {
                                if let Some(first) = first_selected {
                                    if first > 0 {
                                        let _ = session.move_page(first, first - 1);
                                        self.organizer_state.select_single(first - 1);
                                    }
                                }
                            }
                            OrganizerAction::MoveRight => {
                                if let Some(first) = first_selected {
                                    if first + 1 < session.page_count() {
                                        let _ = session.move_page(first, first + 1);
                                        self.organizer_state.select_single(first + 1);
                                    }
                                }
                            }
                            OrganizerAction::Undo => {
                                let _ = session.undo();
                            }
                            OrganizerAction::Redo => {
                                let _ = session.redo();
                            }
                        }
                        self.organizer_state.invalidate_doc(doc_id);
                        self.viewer_state.invalidate_doc(doc_id);
                    }

                    if matches!(act, OrganizerAction::Delete) {
                        self.organizer_state.selected_pages.clear();
                    }
                }
            }
        });

        // 7. Modals & Dialogs
        let active_session = self.active_tab.and_then(|idx| self.sessions.get_mut(idx));
        let modal_action = ModalsComponent::render(
            &ctx,
            &mut self.modals_state,
            &mut self.theme,
            &mut self.thumbnails,
            active_session,
        );

        if let Some(act) = modal_action {
            match act {
                ModalAction::Combine(inputs) => {
                    let mut buffers = Vec::new();
                    for p in inputs {
                        if let Ok(b) = std::fs::read(&p) {
                            buffers.push(b);
                        }
                    }
                    if let Ok(merged) = rustpdf_organize::combine_documents(&buffers) {
                        if let Ok(session) = DocumentSession::open_mem(merged, None) {
                            self.sessions.push(session);
                            self.active_tab = Some(self.sessions.len() - 1);
                            self.active_workspace = ActiveWorkspace::Viewer;
                        }
                    }
                }
                ModalAction::Split(n, out_dir) => {
                    if let Some(session) = self.active_session_mut() {
                        if let Ok(chunks) = rustpdf_organize::split_document(
                            &session.raw_bytes,
                            rustpdf_organize::SplitRule::EveryN(n),
                        ) {
                            let _ = std::fs::create_dir_all(&out_dir);
                            for (i, chunk) in chunks.iter().enumerate() {
                                let part_path =
                                    out_dir.join(format!("split_part_{:03}.pdf", i + 1));
                                let _ = rustpdf_edit::save_atomic(&part_path, chunk);
                            }
                        }
                    }
                }
                ModalAction::ExecuteCommand(cmd) => match cmd.as_str() {
                    "Open PDF" => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("PDF", &["pdf"])
                            .pick_file()
                        {
                            self.open_file(path);
                        }
                    }
                    "Save Document" => {
                        if let Some(s) = self.active_session_mut() {
                            if let Some(ref p) = s.file_path.clone() {
                                let _ = s.save_to(p);
                            }
                        }
                    }
                    "Organize Pages" => {
                        self.active_workspace = ActiveWorkspace::Organizer;
                    }
                    "Find in Document" => {
                        self.search_state.open();
                    }
                    "Appearance Settings" => {
                        self.modals_state.show_appearance = true;
                    }
                    "Thumbnail Settings" => {
                        self.modals_state.show_thumbnails = true;
                    }
                    "Document Properties" => {
                        self.modals_state.show_properties = true;
                    }
                    _ => {}
                },
                ModalAction::Notification(message, is_error) => {
                    if is_error {
                        self.notify_error(message);
                    } else {
                        self.notify_success(message);
                        if let Some(session) = self.active_session() {
                            let doc_id = session.id;
                            self.organizer_state.invalidate_doc(doc_id);
                            self.viewer_state.invalidate_doc(doc_id);
                        }
                    }
                }
                ModalAction::DocumentModified => {
                    if let Some(session) = self.active_session() {
                        let doc_id = session.id;
                        self.organizer_state.invalidate_doc(doc_id);
                        self.viewer_state.invalidate_doc(doc_id);
                    }
                }
            }
        }

        self.render_close_confirmations(&ctx);
    }
}
