//! rustpdf-ui/tools_panel — "All tools" workbench drawer matching Acrobat/PdfCraft reference.
//! Supports hierarchical subviews for "Edit a PDF", "Export a PDF", and "Fill & Sign".

use egui::{
    vec2, Align, Color32, CornerRadius, Layout, Rect, RichText, ScrollArea, Stroke, StrokeKind,
    Ui, UiBuilder,
};
use crate::icons::IconKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DrawerPage {
    #[default]
    AllTools,
    EditPdf,
    ExportPdf,
    FillAndSign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolBadge {
    Ready,
    Active,
    M10,
    Available,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolAction {
    CloseDrawer,
    NavigateTo(DrawerPage),
    NavigateBack,

    // Export subview actions
    ExportWord,
    ExportSpreadsheet,
    ExportPresentation,
    ExportImagePng,
    ExportAllImages,
    ExportHtml,
    ExportRtf,
    ExportTextPlain,
    ExportPostScript,

    // Edit a PDF subview actions
    RotatePages,
    InsertPages,
    DeletePages,
    ExtractPages,
    OrganizePages,
    EditTextAndImages,
    AddTextContent,
    AddImageContent,
    HeaderAndFooter,
    Watermark,
    Link,
    BatesNumbering,
    Background,

    // Fill & Sign subview actions
    FillAddText,
    FillCheckmark,
    FillCross,
    FillDot,
    FillLine,
    FillDate,
    SignAddSignature,
    SignAddInitials,

    // Main All Tools actions
    ExportPdf,
    FillAndSign,
    EditPdf,
    CreatePdf,
    CombineFiles,
    AddComments,
    ScanOcr,
    ProtectPdf,
    RedactPdf,
    CompressPdf,
    PrepareForm,
    AddStamp,
    UseCertificate,
    PrintProduction,
    MeasureObjects,
    CompareFiles,
    GuidedActions,
    Accessibility,
    PdfStandards,
    SearchIndex,
    JavaScript,

    // Supporting actions & modals
    OpenOrganize,
    AddStickyNote,
    AddHighlight,
    AddFreehandInk,
    AddShape,
    OpenFormsManager,
    ResetFormFields,
    ExportFormData,
    OpenProtectDialog,
    RemoveProtection,
    OpenLayersDialog,
    AddAttachment,
    OpenCombineDialog,
    OpenSplitDialog,
    OpenPropertiesDialog,
    OpenAppearanceDialog,
    OpenAboutDialog,
}

pub struct ToolsPanelComponent;

impl ToolsPanelComponent {
    pub fn render(
        ui: &mut Ui,
        current_page: &mut DrawerPage,
        _accent: Color32,
        has_session: bool,
        is_edit_active: bool,
    ) -> Option<ToolAction> {
        let mut action = None;

        match current_page {
            DrawerPage::AllTools => {
                Self::render_all_tools(ui, &mut action, has_session);
            }
            DrawerPage::EditPdf => {
                Self::render_edit_pdf(ui, &mut action, has_session, is_edit_active);
            }
            DrawerPage::ExportPdf => {
                Self::render_export_pdf(ui, &mut action, has_session);
            }
            DrawerPage::FillAndSign => {
                Self::render_fill_and_sign(ui, &mut action, has_session);
            }
        }

        // Apply internal navigation transitions if triggered
        if let Some(ToolAction::NavigateTo(page)) = action {
            *current_page = page;
            action = None;
        } else if let Some(ToolAction::NavigateBack) = action {
            *current_page = DrawerPage::AllTools;
            action = None;
        }

        action
    }

    /// Renders the main "All tools" drawer page.
    fn render_all_tools(ui: &mut Ui, action: &mut Option<ToolAction>, has_session: bool) {
        let is_dark = ui.visuals().dark_mode;
        let title_color = if is_dark { Color32::from_rgb(243, 244, 246) } else { Color32::from_rgb(17, 24, 39) };
        let close_color = if is_dark { Color32::from_gray(160) } else { Color32::from_rgb(107, 114, 128) };

        // Drawer Header with "All tools" title and right-aligned '✕' close button
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.label(RichText::new("All tools").size(20.0).strong().color(title_color));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(6.0);
                let close_btn = ui.add(
                    egui::Button::new(RichText::new("✕").size(16.0).color(close_color))
                        .frame(false)
                );
                if close_btn.clicked() {
                    *action = Some(ToolAction::CloseDrawer);
                }
            });
        });
        ui.add_space(10.0);

        ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());

            // 1. Export a PDF (opens Export subview)
            if Self::draw_tool_row(ui, IconKind::ExportPdf, "Export a PDF", None, None, true) {
                *action = Some(ToolAction::NavigateTo(DrawerPage::ExportPdf));
            }

            // 2. Fill & Sign (opens Fill & Sign subview)
            if Self::draw_tool_row(ui, IconKind::FillAndSign, "Fill & Sign", None, None, true) {
                *action = Some(ToolAction::NavigateTo(DrawerPage::FillAndSign));
            }

            // 3. Edit a PDF (opens Edit PDF subview)
            if Self::draw_tool_row(ui, IconKind::EditPdf, "Edit a PDF", None, None, true) {
                *action = Some(ToolAction::NavigateTo(DrawerPage::EditPdf));
            }

            // 4. Create a PDF
            if Self::draw_tool_row(ui, IconKind::CreatePdf, "Create a PDF", None, None, true) {
                *action = Some(ToolAction::CreatePdf);
            }

            // 5. Combine files (with "Available" badge)
            if Self::draw_tool_row(ui, IconKind::CombineFiles, "Combine files", Some(ToolBadge::Available), None, true) {
                *action = Some(ToolAction::CombineFiles);
            }

            // 6. Organize pages
            if Self::draw_tool_row(ui, IconKind::OrganizePages, "Organize pages", None, None, true) {
                *action = Some(ToolAction::OrganizePages);
            }

            // 7. Add comments
            if Self::draw_tool_row(ui, IconKind::AddComments, "Add comments", None, None, true) {
                *action = Some(ToolAction::AddComments);
            }

            // 8. Scan & OCR
            if Self::draw_tool_row(ui, IconKind::ScanOcr, "Scan & OCR", None, None, true) {
                *action = Some(ToolAction::ScanOcr);
            }

            // 9. Protect a PDF
            if Self::draw_tool_row(ui, IconKind::ProtectPdf, "Protect a PDF", None, None, true) {
                *action = Some(ToolAction::ProtectPdf);
            }

            // 10. Redact a PDF
            if Self::draw_tool_row(ui, IconKind::RedactPdf, "Redact a PDF", None, None, true) {
                *action = Some(ToolAction::RedactPdf);
            }

            // 11. Compress a PDF
            if Self::draw_tool_row(ui, IconKind::CompressPdf, "Compress a PDF", None, None, has_session) {
                *action = Some(ToolAction::CompressPdf);
            }

            // 12. Prepare a form
            if Self::draw_tool_row(ui, IconKind::PrepareForm, "Prepare a form", None, None, true) {
                *action = Some(ToolAction::PrepareForm);
            }

            // 13. Add a stamp
            if Self::draw_tool_row(ui, IconKind::AddStamp, "Add a stamp", None, None, true) {
                *action = Some(ToolAction::AddStamp);
            }

            // 14. Use a certificate
            if Self::draw_tool_row(ui, IconKind::UseCertificate, "Use a certificate", None, None, true) {
                *action = Some(ToolAction::UseCertificate);
            }

            // 15. Use print production
            if Self::draw_tool_row(ui, IconKind::PrintProduction, "Use print production", None, None, true) {
                *action = Some(ToolAction::PrintProduction);
            }

            // 16. Measure objects
            if Self::draw_tool_row(ui, IconKind::MeasureObjects, "Measure objects", None, None, true) {
                *action = Some(ToolAction::MeasureObjects);
            }

            // 17. Compare files
            if Self::draw_tool_row(ui, IconKind::CompareFiles, "Compare files", None, None, true) {
                *action = Some(ToolAction::CompareFiles);
            }

            // 18. Use guided actions (with "Available" badge)
            if Self::draw_tool_row(ui, IconKind::GuidedActions, "Use guided actions", Some(ToolBadge::Available), None, true) {
                *action = Some(ToolAction::GuidedActions);
            }

            // 19. Prepare for accessibility
            if Self::draw_tool_row(ui, IconKind::Accessibility, "Prepare for accessibility", None, None, true) {
                *action = Some(ToolAction::Accessibility);
            }

            // 20. Apply PDF standards
            if Self::draw_tool_row(ui, IconKind::PdfStandards, "Apply PDF standards", None, None, true) {
                *action = Some(ToolAction::PdfStandards);
            }

            // 21. Add search index
            if Self::draw_tool_row(ui, IconKind::SearchIndex, "Add search index", None, None, true) {
                *action = Some(ToolAction::SearchIndex);
            }

            // 22. Use JavaScript
            if Self::draw_tool_row(ui, IconKind::JavaScript, "Use JavaScript", None, None, true) {
                *action = Some(ToolAction::JavaScript);
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(6.0);

            // Additional Workbench Utilities
            if Self::draw_tool_row(ui, IconKind::Layers, "Optional Layers (OCG)", None, None, has_session) {
                *action = Some(ToolAction::OpenLayersDialog);
            }
            if Self::draw_tool_row(ui, IconKind::Attachment, "Add Attachment", None, None, has_session) {
                *action = Some(ToolAction::AddAttachment);
            }
            if Self::draw_tool_row(ui, IconKind::Properties, "Document Properties", None, None, true) {
                *action = Some(ToolAction::OpenPropertiesDialog);
            }
            if Self::draw_tool_row(ui, IconKind::Appearance, "Appearance & Themes", None, None, true) {
                *action = Some(ToolAction::OpenAppearanceDialog);
            }
            if Self::draw_tool_row(ui, IconKind::AppLogo, "About RustPdf", None, None, true) {
                *action = Some(ToolAction::OpenAboutDialog);
            }

            ui.add_space(20.0);
        });
    }

    /// Renders the `< Edit a PDF` subview matching user Screenshot 1.
    fn render_edit_pdf(ui: &mut Ui, action: &mut Option<ToolAction>, _has_session: bool, is_edit_active: bool) {
        Self::render_subview_header(ui, action, "Edit a PDF");

        // Informational Notice Banner matching Screenshot 1
        ui.add_space(6.0);
        let banner_rect = ui.allocate_exact_size(vec2(ui.available_width() - 8.0, 50.0), egui::Sense::hover()).0;
        let is_dark = ui.visuals().dark_mode;
        let banner_bg = if is_dark {
            Color32::from_rgb(30, 42, 60)
        } else {
            Color32::from_rgb(224, 236, 255)
        };
        let banner_text = if is_dark {
            Color32::from_rgb(220, 235, 255)
        } else {
            Color32::from_rgb(20, 30, 50)
        };
        ui.painter().rect(
            banner_rect,
            CornerRadius::same(8),
            banner_bg,
            Stroke::NONE,
            StrokeKind::Inside,
        );
        let mut b_child = ui.new_child(UiBuilder::new().max_rect(banner_rect.shrink(8.0)).layout(Layout::top_down(Align::Min)));
        b_child.label(RichText::new("Coming in milestone M7. Items marked Ready work today.").size(12.0).color(banner_text));

        ui.add_space(14.0);

        ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Section: MODIFY PAGE
            Self::draw_section_header(ui, "MODIFY PAGE");

            let magenta = Color32::from_rgb(225, 29, 116);

            if Self::draw_tool_row(ui, IconKind::RotateCw, "Rotate pages", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::RotatePages);
            }
            if Self::draw_tool_row(ui, IconKind::CreatePdf, "Insert pages", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::InsertPages);
            }
            if Self::draw_tool_row(ui, IconKind::Delete, "Delete pages", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::DeletePages);
            }
            if Self::draw_tool_row(ui, IconKind::ExtractPages, "Extract pages", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::ExtractPages);
            }
            if Self::draw_tool_row(ui, IconKind::OrganizePages, "Organize pages", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::OrganizePages);
            }

            ui.add_space(14.0);

            // Section: ADD CONTENT
            Self::draw_section_header(ui, "ADD CONTENT");

            let edit_badge = if is_edit_active { ToolBadge::Active } else { ToolBadge::Ready };
            if Self::draw_tool_row(ui, IconKind::PrepareForm, "Edit text & images", Some(edit_badge), Some(magenta), true) {
                *action = Some(ToolAction::EditTextAndImages);
            }
            if Self::draw_tool_row(ui, IconKind::AddText, "Text", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::AddTextContent);
            }
            if Self::draw_tool_row(ui, IconKind::AddImage, "Image", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::AddImageContent);
            }
            if Self::draw_tool_row(ui, IconKind::HeaderFooter, "Header and footer", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::HeaderAndFooter);
            }
            if Self::draw_tool_row(ui, IconKind::Watermark, "Watermark", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::Watermark);
            }
            if Self::draw_tool_row(ui, IconKind::Link, "Link", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::Link);
            }
            if Self::draw_tool_row(ui, IconKind::BatesNumbering, "Bates numbering", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::BatesNumbering);
            }
            if Self::draw_tool_row(ui, IconKind::Appearance, "Background", Some(ToolBadge::Ready), Some(magenta), true) {
                *action = Some(ToolAction::Background);
            }

            ui.add_space(20.0);
        });
    }

    /// Renders the `< Export a PDF` subview matching user Screenshot 2.
    fn render_export_pdf(ui: &mut Ui, action: &mut Option<ToolAction>, _has_session: bool) {
        Self::render_subview_header(ui, action, "Export a PDF");

        ui.add_space(10.0);

        ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());

            Self::draw_section_header(ui, "EXPORT TO");

            let blue = Color32::from_rgb(37, 99, 235);

            if Self::draw_tool_row(ui, IconKind::Document, "Microsoft Word (.docx)", Some(ToolBadge::Ready), Some(blue), true) {
                *action = Some(ToolAction::ExportWord);
            }
            if Self::draw_tool_row(ui, IconKind::Spreadsheet, "Spreadsheet (.xlsx)", Some(ToolBadge::M10), Some(blue), false) {
                *action = Some(ToolAction::ExportSpreadsheet);
            }
            if Self::draw_tool_row(ui, IconKind::Presentation, "Presentation (.pptx)", Some(ToolBadge::M10), Some(blue), false) {
                *action = Some(ToolAction::ExportPresentation);
            }
            if Self::draw_tool_row(ui, IconKind::AddImage, "Image (PNG)", Some(ToolBadge::Ready), Some(blue), true) {
                *action = Some(ToolAction::ExportImagePng);
            }
            if Self::draw_tool_row(ui, IconKind::Duplicate, "Export all images", Some(ToolBadge::Ready), Some(blue), true) {
                *action = Some(ToolAction::ExportAllImages);
            }
            if Self::draw_tool_row(ui, IconKind::ExportPdf, "HTML web page", Some(ToolBadge::Ready), Some(blue), true) {
                *action = Some(ToolAction::ExportHtml);
            }
            if Self::draw_tool_row(ui, IconKind::Document, "Rich Text Format (.rtf)", Some(ToolBadge::Ready), Some(blue), true) {
                *action = Some(ToolAction::ExportRtf);
            }
            if Self::draw_tool_row(ui, IconKind::AddText, "Text (plain)", Some(ToolBadge::Ready), Some(blue), true) {
                *action = Some(ToolAction::ExportTextPlain);
            }
            if Self::draw_tool_row(ui, IconKind::CompressPdf, "PostScript / EPS", Some(ToolBadge::M10), Some(blue), false) {
                *action = Some(ToolAction::ExportPostScript);
            }

            ui.add_space(20.0);
        });
    }

    /// Renders the `< Fill & Sign` subview matching user Screenshot 3.
    fn render_fill_and_sign(ui: &mut Ui, action: &mut Option<ToolAction>, _has_session: bool) {
        Self::render_subview_header(ui, action, "Fill & Sign");

        ui.add_space(10.0);

        ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Section: FILL
            Self::draw_section_header(ui, "FILL");

            let purple = Color32::from_rgb(139, 92, 246);

            if Self::draw_tool_row(ui, IconKind::AddText, "Add text", Some(ToolBadge::Ready), Some(purple), true) {
                *action = Some(ToolAction::FillAddText);
            }
            if Self::draw_tool_row(ui, IconKind::Check, "Checkmark", Some(ToolBadge::Ready), Some(purple), true) {
                *action = Some(ToolAction::FillCheckmark);
            }
            if Self::draw_tool_row(ui, IconKind::Close, "Cross", Some(ToolBadge::Ready), Some(purple), true) {
                *action = Some(ToolAction::FillCross);
            }
            if Self::draw_tool_row(ui, IconKind::Dot, "Dot", Some(ToolBadge::Ready), Some(purple), true) {
                *action = Some(ToolAction::FillDot);
            }
            if Self::draw_tool_row(ui, IconKind::Line, "Line", Some(ToolBadge::Ready), Some(purple), true) {
                *action = Some(ToolAction::FillLine);
            }
            if Self::draw_tool_row(ui, IconKind::Date, "Date", Some(ToolBadge::Ready), Some(purple), true) {
                *action = Some(ToolAction::FillDate);
            }

            ui.add_space(16.0);

            // Section: SIGN YOURSELF
            Self::draw_section_header(ui, "SIGN YOURSELF");

            if Self::draw_action_button(ui, "+", "Add signature") {
                *action = Some(ToolAction::SignAddSignature);
            }
            if Self::draw_action_button(ui, "+", "Add initials") {
                *action = Some(ToolAction::SignAddInitials);
            }

            ui.add_space(20.0);
        });
    }

    /// Renders subview back button `<` and title with close `✕`.
    fn render_subview_header(ui: &mut Ui, action: &mut Option<ToolAction>, title: &str) {
        let is_dark = ui.visuals().dark_mode;
        let title_color = if is_dark { Color32::from_rgb(243, 244, 246) } else { Color32::from_rgb(17, 24, 39) };
        let close_color = if is_dark { Color32::from_gray(160) } else { Color32::from_rgb(107, 114, 128) };
        let chevron_color = if is_dark { Color32::from_gray(220) } else { Color32::from_rgb(55, 65, 81) };

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);

            // Rounded Back Button `<`
            let back_rect = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::click()).0;
            let is_back_hovered = ui.rect_contains_pointer(back_rect);
            let back_bg = if is_back_hovered {
                if is_dark { Color32::from_rgb(45, 50, 64) } else { Color32::from_rgb(230, 234, 242) }
            } else {
                if is_dark { Color32::from_rgb(32, 35, 44) } else { Color32::from_rgb(243, 244, 248) }
            };
            ui.painter().rect(
                back_rect,
                CornerRadius::same(6),
                back_bg,
                Stroke::NONE,
                StrokeKind::Inside,
            );
            let mut back_child = ui.new_child(UiBuilder::new().max_rect(back_rect).layout(Layout::centered_and_justified(egui::Direction::LeftToRight)));
            if back_child.label(RichText::new("‹").size(20.0).strong().color(chevron_color)).clicked() || (is_back_hovered && ui.input(|i| i.pointer.primary_clicked())) {
                *action = Some(ToolAction::NavigateBack);
            }

            ui.add_space(8.0);
            ui.label(RichText::new(title).size(19.0).strong().color(title_color));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(6.0);
                let close_btn = ui.add(
                    egui::Button::new(RichText::new("✕").size(16.0).color(close_color))
                        .frame(false)
                );
                if close_btn.clicked() {
                    *action = Some(ToolAction::CloseDrawer);
                }
            });
        });
        ui.add_space(10.0);
    }

    /// Renders clean uppercase section headers matching the screenshots (e.g. "MODIFY PAGE").
    fn draw_section_header(ui: &mut Ui, title: &str) {
        let is_dark = ui.visuals().dark_mode;
        let color = if is_dark { Color32::from_gray(150) } else { Color32::from_rgb(107, 114, 128) };
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(RichText::new(title).size(11.0).strong().color(color));
        });
        ui.add_space(4.0);
    }

    /// Renders an action button with leading symbol like "+ Add signature".
    fn draw_action_button(ui: &mut Ui, symbol: &str, label: &str) -> bool {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), egui::Sense::click());
        let is_hovered = resp.hovered();
        let is_dark = ui.visuals().dark_mode;

        if is_hovered {
            let bg = if is_dark { Color32::from_rgb(38, 42, 54) } else { Color32::from_rgb(243, 244, 246) };
            ui.painter().rect(
                rect.shrink2(vec2(4.0, 1.0)),
                CornerRadius::same(6),
                bg,
                Stroke::NONE,
                StrokeKind::Inside,
            );
        }

        let symbol_color = if is_dark { Color32::from_gray(210) } else { Color32::from_rgb(55, 65, 81) };
        let text_color = if is_dark { Color32::from_rgb(243, 244, 246) } else { Color32::from_rgb(17, 24, 39) };

        let mut child = ui.new_child(UiBuilder::new().max_rect(rect.shrink2(vec2(12.0, 0.0))).layout(Layout::left_to_right(Align::Center)));
        child.label(RichText::new(symbol).size(18.0).strong().color(symbol_color));
        child.add_space(8.0);
        child.label(RichText::new(label).size(13.5).strong().color(text_color));

        resp.clicked()
    }

    /// Renders a single tool row matching the visual identity of the reference screenshots.
    fn draw_tool_row(
        ui: &mut Ui,
        icon: IconKind,
        title: &str,
        badge: Option<ToolBadge>,
        override_color: Option<Color32>,
        enabled: bool,
    ) -> bool {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), egui::Sense::click());

        let is_hovered = resp.hovered() && enabled;
        let is_dark = ui.visuals().dark_mode;

        let bg_color = if is_hovered {
            if is_dark {
                Color32::from_rgb(38, 42, 54)
            } else {
                Color32::from_rgb(243, 244, 246)
            }
        } else {
            Color32::TRANSPARENT
        };

        if bg_color != Color32::TRANSPARENT {
            ui.painter().rect(
                rect.shrink2(vec2(4.0, 1.0)),
                CornerRadius::same(6),
                bg_color,
                Stroke::NONE,
                StrokeKind::Inside,
            );
        }

        // Draw tool icon using signature color or subview override color
        let icon_color = if enabled {
            override_color.unwrap_or_else(|| icon.default_color())
        } else {
            Color32::from_gray(100)
        };

        let icon_rect = Rect::from_center_size(
            egui::pos2(rect.min.x + 22.0, rect.center().y),
            vec2(20.0, 20.0),
        );
        crate::icons::Icons::draw_icon(ui.painter(), icon_rect, icon, icon_color);

        // Tool Title Label
        let text_color = if enabled {
            if is_dark {
                Color32::from_rgb(230, 233, 240)
            } else {
                Color32::from_rgb(20, 24, 33)
            }
        } else {
            Color32::from_gray(120)
        };

        let badge_offset = if badge.is_some() { 68.0 } else { 20.0 };
        let title_rect = Rect::from_min_max(
            egui::pos2(rect.min.x + 44.0, rect.min.y),
            egui::pos2(rect.max.x - badge_offset, rect.max.y),
        );
        let mut child = ui.new_child(UiBuilder::new().max_rect(title_rect).layout(Layout::left_to_right(Align::Center)));
        child.label(RichText::new(title).size(13.5).strong().color(text_color));

        // Pill badge on the right
        if let Some(b) = badge {
            if b != ToolBadge::Available || is_hovered {
                let badge_w = match b {
                    ToolBadge::M10 => 36.0,
                    ToolBadge::Ready => 48.0,
                    ToolBadge::Active => 52.0,
                    ToolBadge::Available => 64.0,
                };
                let badge_h = 20.0;
                let badge_rect = Rect::from_min_size(
                    egui::pos2(rect.max.x - badge_w - 8.0, rect.center().y - badge_h * 0.5),
                    vec2(badge_w, badge_h),
                );

                let (badge_bg, badge_text_color, label) = match b {
                    ToolBadge::Active => {
                        let bg = if is_dark { Color32::from_rgb(30, 58, 138) } else { Color32::from_rgb(219, 234, 254) };
                        let txt = if is_dark { Color32::from_rgb(96, 165, 250) } else { Color32::from_rgb(29, 78, 216) };
                        (bg, txt, "Active")
                    }
                    ToolBadge::Ready => {
                        let bg = if is_dark { Color32::from_rgb(26, 52, 40) } else { Color32::from_rgb(220, 252, 231) };
                        let txt = if is_dark { Color32::from_rgb(74, 222, 128) } else { Color32::from_rgb(21, 128, 61) };
                        (bg, txt, "Ready")
                    }
                    ToolBadge::M10 => {
                        let bg = if is_dark { Color32::from_rgb(45, 50, 60) } else { Color32::from_rgb(235, 238, 244) };
                        let txt = if is_dark { Color32::from_rgb(160, 170, 185) } else { Color32::from_rgb(100, 116, 139) };
                        (bg, txt, "M10")
                    }
                    ToolBadge::Available => {
                        let bg = if is_dark { Color32::from_rgb(26, 29, 36) } else { Color32::from_rgb(255, 255, 255) };
                        let txt = Color32::from_gray(170);
                        (bg, txt, "Available")
                    }
                };

                ui.painter().rect(
                    badge_rect,
                    CornerRadius::same(5),
                    badge_bg,
                    Stroke::NONE,
                    StrokeKind::Inside,
                );

                let mut b_child = ui.new_child(UiBuilder::new().max_rect(badge_rect).layout(Layout::centered_and_justified(egui::Direction::LeftToRight)));
                b_child.label(RichText::new(label).size(10.5).strong().color(badge_text_color));
            }
        }

        enabled && resp.clicked()
    }
}
