//! rustpdf-ui/icons — Embedded vector icons and branding assets.

use egui::{pos2, vec2, Color32, CornerRadius, Painter, Rect, Stroke, StrokeKind, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconKind {
    AppLogo,
    ExportPdf,
    FillAndSign,
    EditPdf,
    CreatePdf,
    CombineFiles,
    OrganizePages,
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
    FolderOpen,
    AddText,
    AddImage,
    HeaderFooter,
    Watermark,
    Link,
    BatesNumbering,
    Spreadsheet,
    Presentation,
    Dot,
    Line,
    Date,
    ExtractPages,

    // Aliases & workspace utility icons
    Document,
    NewBlank,
    Save,
    Organize,
    Combine,
    Split,
    RotateCw,
    RotateCcw,
    Delete,
    Duplicate,
    StickyNote,
    Highlight,
    FreehandInk,
    Shape,
    Forms,
    ResetForm,
    ExportData,
    Protect,
    Unlock,
    Layers,
    Attachment,
    Properties,
    Appearance,
    Thumbnails,
    Search,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    Close,
    Check,
    About,
}

impl IconKind {
    /// Returns the distinctive, iconic stroke color for each tool matching modern PDF workbenches.
    pub fn default_color(&self) -> Color32 {
        match self {
            IconKind::ExportPdf | IconKind::ExportData => Color32::from_rgb(37, 99, 235), // Blue
            IconKind::FillAndSign | IconKind::FreehandInk => Color32::from_rgb(139, 92, 246), // Purple
            IconKind::EditPdf => Color32::from_rgb(225, 29, 116), // Magenta
            IconKind::CreatePdf | IconKind::NewBlank => Color32::from_rgb(239, 68, 68), // Red
            IconKind::CombineFiles | IconKind::Combine => Color32::from_rgb(59, 130, 246), // Blue
            IconKind::OrganizePages | IconKind::Organize => Color32::from_rgb(16, 185, 129), // Emerald Green
            IconKind::AddComments | IconKind::StickyNote => Color32::from_rgb(245, 158, 11), // Amber Orange
            IconKind::ScanOcr => Color32::from_rgb(20, 184, 166), // Mint Teal
            IconKind::ProtectPdf | IconKind::Protect => Color32::from_rgb(37, 99, 235), // Shield Blue
            IconKind::RedactPdf => Color32::from_rgb(220, 38, 38), // Crimson Red
            IconKind::CompressPdf => Color32::from_rgb(239, 68, 68), // Red
            IconKind::PrepareForm | IconKind::Forms => Color32::from_rgb(147, 51, 234), // Purple
            IconKind::AddStamp => Color32::from_rgb(168, 85, 247), // Purple
            IconKind::UseCertificate => Color32::from_rgb(6, 182, 212), // Cyan
            IconKind::PrintProduction => Color32::from_rgb(124, 58, 237), // Indigo Purple
            IconKind::MeasureObjects => Color32::from_rgb(236, 72, 153), // Rose Pink
            IconKind::CompareFiles => Color32::from_rgb(217, 70, 239), // Magenta
            IconKind::GuidedActions => Color32::from_rgb(139, 92, 246), // Purple
            IconKind::Accessibility => Color32::from_rgb(59, 130, 246), // Blue
            IconKind::PdfStandards => Color32::from_rgb(239, 68, 68), // Red
            IconKind::SearchIndex | IconKind::Search => Color32::from_rgb(16, 185, 129), // Green
            IconKind::JavaScript => Color32::from_rgb(37, 99, 235), // Blue
            IconKind::FolderOpen => Color32::from_rgb(100, 116, 139), // Slate
            IconKind::AddText | IconKind::Dot | IconKind::Line | IconKind::Date => Color32::from_rgb(139, 92, 246), // Purple
            IconKind::HeaderFooter | IconKind::Watermark | IconKind::Link | IconKind::BatesNumbering | IconKind::ExtractPages => Color32::from_rgb(225, 29, 116), // Magenta
            IconKind::Spreadsheet | IconKind::Presentation | IconKind::AddImage => Color32::from_rgb(37, 99, 235), // Blue
            IconKind::AppLogo | IconKind::About => Color32::from_rgb(235, 95, 38), // Rust Orange
            _ => Color32::from_gray(180),
        }
    }
}

pub struct Icons;

impl Icons {
    /// Embedded scalable vector SVG of the RustPdf logo.
    pub const LOGO_SVG: &'static str = include_str!("../../../assets/branding/rustpdf_logo.svg");

    /// Embedded raster PNG of the RustPdf app logo.
    pub const LOGO_PNG: &'static [u8] = include_bytes!("../../../assets/branding/rustpdf_logo.png");

    // Embedded Vector Tool Icons
    pub const ICON_DOCUMENT: &'static str = include_str!("../../../assets/icons/document.svg");
    pub const ICON_BLANK: &'static str = include_str!("../../../assets/icons/blank.svg");
    pub const ICON_SAVE: &'static str = include_str!("../../../assets/icons/save.svg");
    pub const ICON_ROTATE_CW: &'static str = include_str!("../../../assets/icons/rotate_cw.svg");
    pub const ICON_ROTATE_CCW: &'static str = include_str!("../../../assets/icons/rotate_ccw.svg");
    pub const ICON_DELETE: &'static str = include_str!("../../../assets/icons/delete.svg");
    pub const ICON_DUPLICATE: &'static str = include_str!("../../../assets/icons/duplicate.svg");
    pub const ICON_COMBINE: &'static str = include_str!("../../../assets/icons/combine.svg");
    pub const ICON_SPLIT: &'static str = include_str!("../../../assets/icons/split.svg");
    pub const ICON_ORGANIZE: &'static str = include_str!("../../../assets/icons/organize.svg");
    pub const ICON_SEARCH: &'static str = include_str!("../../../assets/icons/search.svg");
    pub const ICON_PALETTE: &'static str = include_str!("../../../assets/icons/palette.svg");
    pub const ICON_THUMBNAILS: &'static str = include_str!("../../../assets/icons/thumbnails.svg");
    pub const ICON_BOOKMARK: &'static str = include_str!("../../../assets/icons/bookmark.svg");
    pub const ICON_PROPERTIES: &'static str = include_str!("../../../assets/icons/properties.svg");
    pub const ICON_UNDO: &'static str = include_str!("../../../assets/icons/undo.svg");
    pub const ICON_REDO: &'static str = include_str!("../../../assets/icons/redo.svg");
    pub const ICON_ZOOM_IN: &'static str = include_str!("../../../assets/icons/zoom_in.svg");
    pub const ICON_ZOOM_OUT: &'static str = include_str!("../../../assets/icons/zoom_out.svg");

    /// Draws the distinctive RustPdf brand logo with rich geometry, layered sheets, and vibrant fold accent.
    pub fn draw_logo(painter: &Painter, rect: Rect, primary_color: Color32) {
        let w = rect.width();
        let h = rect.height();
        let center = rect.center();

        // 1. Dark Slate Foundation Sheet (Backdrop Card)
        let card_w = w * 0.90;
        let card_h = h * 0.94;
        let card_rect = Rect::from_center_size(center, vec2(card_w, card_h));
        let corner_radius = (w * 0.18).clamp(3.0, 16.0);

        // Foundation drop shadow
        painter.rect(
            card_rect.translate(vec2(0.0, h * 0.04)),
            CornerRadius::same(corner_radius as u8),
            Color32::from_black_alpha(50),
            Stroke::NONE,
            StrokeKind::Inside,
        );

        // Foundation body
        painter.rect(
            card_rect,
            CornerRadius::same(corner_radius as u8),
            Color32::from_rgb(32, 35, 42),
            Stroke::new(1.0, Color32::from_rgb(48, 52, 64)),
            StrokeKind::Inside,
        );

        // 2. Main Crisp Document Polygon with Top-Right Origami Fold
        let doc_w = w * 0.64;
        let doc_h = h * 0.72;
        let left = center.x - doc_w * 0.5;
        let right = center.x + doc_w * 0.5;
        let top = center.y - doc_h * 0.5 - h * 0.02;
        let bottom = center.y + doc_h * 0.5 - h * 0.02;

        let fold_size = (doc_w * 0.38).min(doc_h * 0.38);

        let sheet_points = vec![
            pos2(left, top),
            pos2(right - fold_size, top),
            pos2(right, top + fold_size),
            pos2(right, bottom),
            pos2(left, bottom),
        ];

        painter.add(egui::Shape::convex_polygon(
            sheet_points,
            Color32::from_rgb(248, 250, 253),
            Stroke::new(1.2, Color32::from_gray(210)),
        ));

        // 3. Origami Folded Corner (Rust/Orange Gradient Accent)
        let fold_points = vec![
            pos2(right - fold_size, top),
            pos2(right - fold_size, top + fold_size),
            pos2(right, top + fold_size),
        ];

        let rust_accent = Color32::from_rgb(235, 95, 38);
        painter.add(egui::Shape::convex_polygon(
            fold_points,
            rust_accent,
            Stroke::new(1.0, Color32::from_rgb(255, 140, 70)),
        ));

        // Fold inner shadow crease
        let fold_shadow = vec![
            pos2(right - fold_size, top + fold_size),
            pos2(right - fold_size - doc_w * 0.12, top + fold_size + doc_h * 0.08),
            pos2(right - fold_size + doc_w * 0.05, top + fold_size + doc_h * 0.10),
        ];
        painter.add(egui::Shape::convex_polygon(
            fold_shadow,
            Color32::from_black_alpha(35),
            Stroke::NONE,
        ));

        // 4. Document Graphic Accent Lines
        let line_x1 = left + doc_w * 0.18;
        let line_x2 = right - doc_w * 0.18;

        // Header accent bar (matches active theme accent or vibrant rust)
        let header_y = top + doc_h * 0.44;
        let header_rect = Rect::from_min_max(
            pos2(line_x1, header_y - doc_h * 0.04),
            pos2(line_x1 + (line_x2 - line_x1) * 0.70, header_y + doc_h * 0.04),
        );
        painter.rect_filled(header_rect, CornerRadius::same(2), primary_color);

        // Body content line 1
        let y2 = top + doc_h * 0.62;
        painter.line_segment(
            [pos2(line_x1, y2), pos2(line_x2, y2)],
            Stroke::new(1.4, Color32::from_gray(160)),
        );

        // Body content line 2
        let y3 = top + doc_h * 0.76;
        painter.line_segment(
            [pos2(line_x1, y3), pos2(line_x1 + (line_x2 - line_x1) * 0.55, y3)],
            Stroke::new(1.4, Color32::from_gray(180)),
        );
    }

    /// Draws a crisp, professional vector stroke icon for any tool or workspace action.
    pub fn draw_icon(painter: &Painter, rect: Rect, kind: IconKind, stroke_color: Color32) {
        let p = rect.min;
        let w = rect.width();
        let h = rect.height();
        let stroke = Stroke::new((w * 0.08).clamp(1.3, 2.2), stroke_color);

        match kind {
            IconKind::AppLogo => {
                Self::draw_logo(painter, rect, stroke_color);
            }

            IconKind::Document => {
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.80;
                let top = p.y + h * 0.12;
                let bottom = p.y + h * 0.88;
                let fold = w * 0.22;

                let pts = vec![
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bottom),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                // Fold flap
                painter.line_segment([pos2(right - fold, top), pos2(right - fold, top + fold)], stroke);
                painter.line_segment([pos2(right - fold, top + fold), pos2(right, top + fold)], stroke);
                // Content lines
                painter.line_segment([pos2(left + w * 0.12, top + h * 0.42), pos2(right - w * 0.12, top + h * 0.42)], stroke);
                painter.line_segment([pos2(left + w * 0.12, top + h * 0.58), pos2(right - w * 0.12, top + h * 0.58)], stroke);
            }

            IconKind::NewBlank => {
                let left = p.x + w * 0.22;
                let right = p.x + w * 0.78;
                let top = p.y + h * 0.12;
                let bottom = p.y + h * 0.88;
                let fold = w * 0.20;

                let pts = vec![
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bottom),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                // Center plus
                let cx = (left + right) * 0.5;
                let cy = (top + bottom) * 0.55;
                let r = w * 0.14;
                painter.line_segment([pos2(cx - r, cy), pos2(cx + r, cy)], stroke);
                painter.line_segment([pos2(cx, cy - r), pos2(cx, cy + r)], stroke);
            }

            IconKind::Save => {
                let left = p.x + w * 0.18;
                let right = p.x + w * 0.82;
                let top = p.y + h * 0.18;
                let bottom = p.y + h * 0.82;
                let cut = w * 0.14;

                let pts = vec![
                    pos2(left, top),
                    pos2(right - cut, top),
                    pos2(right, top + cut),
                    pos2(right, bottom),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                // Top shutter
                painter.rect_stroke(
                    Rect::from_min_max(pos2(left + w * 0.12, top), pos2(right - w * 0.22, top + h * 0.25)),
                    CornerRadius::ZERO,
                    stroke,
                    StrokeKind::Middle,
                );
                // Bottom label
                painter.rect_stroke(
                    Rect::from_min_max(pos2(left + w * 0.10, bottom - h * 0.32), pos2(right - w * 0.10, bottom)),
                    CornerRadius::ZERO,
                    stroke,
                    StrokeKind::Middle,
                );
            }

            IconKind::Organize => {
                // 4-Quadrant Page Tiles
                let gap = w * 0.08;
                let tw = (w * 0.74 - gap) * 0.5;
                let th = (h * 0.74 - gap) * 0.5;
                let start_x = p.x + w * 0.13;
                let start_y = p.y + h * 0.13;

                for row in 0..2 {
                    for col in 0..2 {
                        let rx = start_x + (col as f32) * (tw + gap);
                        let ry = start_y + (row as f32) * (th + gap);
                        let r = Rect::from_min_size(pos2(rx, ry), vec2(tw, th));
                        let s = if row == 0 && col == 1 {
                            Stroke::new(stroke.width, stroke_color)
                        } else {
                            Stroke::new(stroke.width * 0.85, stroke_color)
                        };
                        painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
                    }
                }
            }

            IconKind::Combine => {
                // Stacked documents with plus
                let r1 = Rect::from_min_size(pos2(p.x + w * 0.15, p.y + h * 0.22), vec2(w * 0.44, h * 0.58));
                let r2 = Rect::from_min_size(pos2(p.x + w * 0.35, p.y + h * 0.12), vec2(w * 0.44, h * 0.58));
                painter.rect_stroke(r2, CornerRadius::same(2), stroke, StrokeKind::Middle);
                painter.rect_stroke(r1, CornerRadius::same(2), stroke, StrokeKind::Middle);
                // Mini plus in bottom-right
                let cx = p.x + w * 0.76;
                let cy = p.y + h * 0.76;
                let pr = w * 0.12;
                painter.line_segment([pos2(cx - pr, cy), pos2(cx + pr, cy)], stroke);
                painter.line_segment([pos2(cx, cy - pr), pos2(cx, cy + pr)], stroke);
            }

            IconKind::Split => {
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.80;
                let top = p.y + h * 0.14;
                let bottom = p.y + h * 0.86;
                painter.rect_stroke(Rect::from_min_max(pos2(left, top), pos2(right, bottom)), CornerRadius::same(2), stroke, StrokeKind::Middle);
                // Dividing dashed center cut
                let mid_y = (top + bottom) * 0.5;
                painter.line_segment([pos2(left, mid_y), pos2(left + w * 0.14, mid_y)], stroke);
                painter.line_segment([pos2(left + w * 0.24, mid_y), pos2(right - w * 0.24, mid_y)], stroke);
                painter.line_segment([pos2(right - w * 0.14, mid_y), pos2(right, mid_y)], stroke);
            }

            IconKind::RotateCw => {
                let center = rect.center();
                let r = w * 0.30;
                // Circular arc
                let mut pts = Vec::new();
                for i in 0..16 {
                    let angle = (i as f32) * 2.0 * std::f32::consts::PI * 0.72 / 15.0 - std::f32::consts::PI * 0.5;
                    pts.push(pos2(center.x + angle.cos() * r, center.y + angle.sin() * r));
                }
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                // Arrow tip
                let end = *pts.last().unwrap();
                painter.line_segment([end, pos2(end.x + w * 0.10, end.y - h * 0.08)], stroke);
                painter.line_segment([end, pos2(end.x - w * 0.04, end.y - h * 0.12)], stroke);
            }

            IconKind::RotateCcw => {
                let center = rect.center();
                let r = w * 0.30;
                let mut pts = Vec::new();
                for i in 0..16 {
                    let angle = std::f32::consts::PI * 1.5 - (i as f32) * 2.0 * std::f32::consts::PI * 0.72 / 15.0;
                    pts.push(pos2(center.x + angle.cos() * r, center.y + angle.sin() * r));
                }
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                let end = *pts.last().unwrap();
                painter.line_segment([end, pos2(end.x - w * 0.10, end.y - h * 0.08)], stroke);
                painter.line_segment([end, pos2(end.x + w * 0.04, end.y - h * 0.12)], stroke);
            }

            IconKind::Delete => {
                let left = p.x + w * 0.24;
                let right = p.x + w * 0.76;
                let top = p.y + h * 0.28;
                let bottom = p.y + h * 0.84;
                // Lid
                painter.line_segment([pos2(p.x + w * 0.16, top), pos2(p.x + w * 0.84, top)], stroke);
                painter.line_segment([pos2(p.x + w * 0.40, top - h * 0.08), pos2(p.x + w * 0.60, top - h * 0.08)], stroke);
                // Bin
                let pts = vec![
                    pos2(left, top),
                    pos2(left + w * 0.06, bottom),
                    pos2(right - w * 0.06, bottom),
                    pos2(right, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                // Slats
                painter.line_segment([pos2(p.x + w * 0.42, top + h * 0.12), pos2(p.x + w * 0.42, bottom - h * 0.10)], stroke);
                painter.line_segment([pos2(p.x + w * 0.58, top + h * 0.12), pos2(p.x + w * 0.58, bottom - h * 0.10)], stroke);
            }

            IconKind::Duplicate => {
                let r1 = Rect::from_min_size(pos2(p.x + w * 0.16, p.y + h * 0.26), vec2(w * 0.48, h * 0.58));
                let r2 = Rect::from_min_size(pos2(p.x + w * 0.34, p.y + h * 0.14), vec2(w * 0.48, h * 0.58));
                painter.rect_stroke(r2, CornerRadius::same(2), stroke, StrokeKind::Middle);
                painter.rect_stroke(r1, CornerRadius::same(2), stroke, StrokeKind::Middle);
            }

            IconKind::StickyNote => {
                let left = p.x + w * 0.18;
                let right = p.x + w * 0.82;
                let top = p.y + h * 0.18;
                let bottom = p.y + h * 0.82;
                let fold = w * 0.22;

                let pts = vec![
                    pos2(left, top),
                    pos2(right, top),
                    pos2(right, bottom - fold),
                    pos2(right - fold, bottom),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                painter.line_segment([pos2(right - fold, bottom), pos2(right - fold, bottom - fold)], stroke);
                painter.line_segment([pos2(right - fold, bottom - fold), pos2(right, bottom - fold)], stroke);
                // Lines
                painter.line_segment([pos2(left + w * 0.12, top + h * 0.24), pos2(right - w * 0.16, top + h * 0.24)], stroke);
                painter.line_segment([pos2(left + w * 0.12, top + h * 0.42), pos2(right - w * 0.24, top + h * 0.42)], stroke);
            }

            IconKind::Highlight => {
                // Slanted marker pen
                let tip = pos2(p.x + w * 0.24, p.y + h * 0.74);
                let top_right = pos2(p.x + w * 0.76, p.y + h * 0.22);
                painter.line_segment([tip, top_right], Stroke::new(stroke.width * 2.2, stroke_color));
                // Chisel tip
                painter.line_segment([pos2(p.x + w * 0.18, p.y + h * 0.78), tip], stroke);
                // Neon streak below
                painter.line_segment([pos2(p.x + w * 0.14, p.y + h * 0.86), pos2(p.x + w * 0.65, p.y + h * 0.86)], Stroke::new(stroke.width * 1.5, stroke_color));
            }

            IconKind::FreehandInk => {
                // Pen nib with ink wave
                let cx = rect.center().x;
                let pts = vec![
                    pos2(cx - w * 0.20, p.y + h * 0.16),
                    pos2(cx + w * 0.20, p.y + h * 0.16),
                    pos2(cx + w * 0.14, p.y + h * 0.48),
                    pos2(cx, p.y + h * 0.68),
                    pos2(cx - w * 0.14, p.y + h * 0.48),
                    pos2(cx - w * 0.20, p.y + h * 0.16),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                painter.line_segment([pos2(cx, p.y + h * 0.24), pos2(cx, p.y + h * 0.52)], stroke);
                // Ink curve below
                painter.line_segment([pos2(p.x + w * 0.18, p.y + h * 0.84), pos2(p.x + w * 0.50, p.y + h * 0.78)], stroke);
                painter.line_segment([pos2(p.x + w * 0.50, p.y + h * 0.78), pos2(p.x + w * 0.82, p.y + h * 0.86)], stroke);
            }

            IconKind::Shape => {
                // Rectangle overlapping circle
                painter.rect_stroke(Rect::from_min_size(pos2(p.x + w * 0.16, p.y + h * 0.16), vec2(w * 0.42, h * 0.42)), CornerRadius::same(2), stroke, StrokeKind::Middle);
                painter.circle_stroke(pos2(p.x + w * 0.64, p.y + h * 0.64), w * 0.22, stroke);
            }

            IconKind::Forms => {
                // Clipboard with checkmark
                let left = p.x + w * 0.22;
                let right = p.x + w * 0.78;
                let top = p.y + h * 0.20;
                let bottom = p.y + h * 0.86;
                painter.rect_stroke(Rect::from_min_max(pos2(left, top), pos2(right, bottom)), CornerRadius::same(3), stroke, StrokeKind::Middle);
                // Top clip clamp
                painter.rect_stroke(Rect::from_min_max(pos2(p.x + w * 0.36, p.y + h * 0.12), pos2(p.x + w * 0.64, top + h * 0.04)), CornerRadius::same(2), stroke, StrokeKind::Middle);
                // Checkmark inside
                let c1 = pos2(left + w * 0.14, top + h * 0.38);
                let c2 = pos2(left + w * 0.24, top + h * 0.48);
                let c3 = pos2(right - w * 0.14, top + h * 0.30);
                painter.line_segment([c1, c2], stroke);
                painter.line_segment([c2, c3], stroke);
                // Content line
                painter.line_segment([pos2(left + w * 0.14, top + h * 0.58), pos2(right - w * 0.14, top + h * 0.58)], stroke);
            }

            IconKind::ResetForm => {
                // Circular refresh cycle
                let center = rect.center();
                let r = w * 0.30;
                let mut pts = Vec::new();
                for i in 0..16 {
                    let angle = (i as f32) * 2.0 * std::f32::consts::PI * 0.75 / 15.0;
                    pts.push(pos2(center.x + angle.cos() * r, center.y + angle.sin() * r));
                }
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                let end = *pts.last().unwrap();
                painter.line_segment([end, pos2(end.x - w * 0.10, end.y - h * 0.08)], stroke);
                painter.line_segment([end, pos2(end.x + w * 0.02, end.y - h * 0.12)], stroke);
            }

            IconKind::ExportData => {
                // Outward tray box with up arrow
                let left = p.x + w * 0.18;
                let right = p.x + w * 0.82;
                let mid_y = p.y + h * 0.52;
                let bottom = p.y + h * 0.84;
                painter.line_segment([pos2(left, mid_y), pos2(left, bottom)], stroke);
                painter.line_segment([pos2(left, bottom), pos2(right, bottom)], stroke);
                painter.line_segment([pos2(right, bottom), pos2(right, mid_y)], stroke);
                // Arrow pointing out
                let cx = rect.center().x;
                painter.line_segment([pos2(cx, p.y + h * 0.18), pos2(cx, p.y + h * 0.62)], stroke);
                painter.line_segment([pos2(cx, p.y + h * 0.18), pos2(cx - w * 0.14, p.y + h * 0.32)], stroke);
                painter.line_segment([pos2(cx, p.y + h * 0.18), pos2(cx + w * 0.14, p.y + h * 0.32)], stroke);
            }

            IconKind::Protect => {
                // Padlock
                let body = Rect::from_min_max(pos2(p.x + w * 0.22, p.y + h * 0.44), pos2(p.x + w * 0.78, p.y + h * 0.86));
                painter.rect_stroke(body, CornerRadius::same(3), stroke, StrokeKind::Middle);
                // Locked shackle arch
                let shackle_left = p.x + w * 0.32;
                let shackle_right = p.x + w * 0.68;
                let shackle_top = p.y + h * 0.16;
                painter.line_segment([pos2(shackle_left, p.y + h * 0.44), pos2(shackle_left, shackle_top + h * 0.10)], stroke);
                painter.line_segment([pos2(shackle_left, shackle_top + h * 0.10), pos2(shackle_right, shackle_top + h * 0.10)], stroke);
                painter.line_segment([pos2(shackle_right, shackle_top + h * 0.10), pos2(shackle_right, p.y + h * 0.44)], stroke);
                // Keyhole
                let cx = rect.center().x;
                painter.circle_stroke(pos2(cx, p.y + h * 0.60), w * 0.05, stroke);
                painter.line_segment([pos2(cx, p.y + h * 0.63), pos2(cx, p.y + h * 0.74)], stroke);
            }

            IconKind::Unlock => {
                let body = Rect::from_min_max(pos2(p.x + w * 0.22, p.y + h * 0.44), pos2(p.x + w * 0.78, p.y + h * 0.86));
                painter.rect_stroke(body, CornerRadius::same(3), stroke, StrokeKind::Middle);
                // Open shackle arch
                let shackle_left = p.x + w * 0.32;
                let shackle_right = p.x + w * 0.68;
                let shackle_top = p.y + h * 0.12;
                painter.line_segment([pos2(shackle_left, p.y + h * 0.44), pos2(shackle_left, shackle_top)], stroke);
                painter.line_segment([pos2(shackle_left, shackle_top), pos2(shackle_right, shackle_top)], stroke);
                painter.line_segment([pos2(shackle_right, shackle_top), pos2(shackle_right, shackle_top + h * 0.14)], stroke);
            }

            IconKind::Layers => {
                // 3 Stacked Isometric Rhombuses
                let cx = rect.center().x;
                let ry1 = p.y + h * 0.24;
                let ry2 = p.y + h * 0.48;
                let ry3 = p.y + h * 0.72;
                let dx = w * 0.36;
                let dy = h * 0.14;

                // Top rhombus
                painter.line_segment([pos2(cx, ry1 - dy), pos2(cx + dx, ry1)], stroke);
                painter.line_segment([pos2(cx + dx, ry1), pos2(cx, ry1 + dy)], stroke);
                painter.line_segment([pos2(cx, ry1 + dy), pos2(cx - dx, ry1)], stroke);
                painter.line_segment([pos2(cx - dx, ry1), pos2(cx, ry1 - dy)], stroke);

                // Middle layer chevrons
                painter.line_segment([pos2(cx - dx, ry2), pos2(cx, ry2 + dy)], stroke);
                painter.line_segment([pos2(cx, ry2 + dy), pos2(cx + dx, ry2)], stroke);

                // Bottom layer chevrons
                painter.line_segment([pos2(cx - dx, ry3), pos2(cx, ry3 + dy)], stroke);
                painter.line_segment([pos2(cx, ry3 + dy), pos2(cx + dx, ry3)], stroke);
            }

            IconKind::Attachment => {
                // Curved paperclip
                let left = p.x + w * 0.30;
                let right = p.x + w * 0.70;
                let top = p.y + h * 0.20;
                let bottom = p.y + h * 0.80;

                painter.line_segment([pos2(left + w * 0.10, top + h * 0.20), pos2(left + w * 0.10, bottom - h * 0.12)], stroke);
                painter.line_segment([pos2(left + w * 0.10, bottom - h * 0.12), pos2(right - w * 0.10, bottom - h * 0.12)], stroke);
                painter.line_segment([pos2(right - w * 0.10, bottom - h * 0.12), pos2(right - w * 0.10, top + h * 0.10)], stroke);
                painter.line_segment([pos2(right - w * 0.10, top + h * 0.10), pos2(left, top + h * 0.10)], stroke);
                painter.line_segment([pos2(left, top + h * 0.10), pos2(left, bottom)], stroke);
                painter.line_segment([pos2(left, bottom), pos2(right, bottom)], stroke);
                painter.line_segment([pos2(right, bottom), pos2(right, top)], stroke);
            }

            IconKind::Properties => {
                let center = rect.center();
                let r = w * 0.36;
                painter.circle_stroke(center, r, stroke);
                // "i" glyph
                painter.circle_filled(pos2(center.x, center.y - h * 0.16), stroke.width * 0.85, stroke_color);
                painter.line_segment([pos2(center.x, center.y - h * 0.05), pos2(center.x, center.y + h * 0.20)], stroke);
            }

            IconKind::Appearance => {
                // Artist palette
                let center = rect.center();
                let r = w * 0.36;
                painter.circle_stroke(center, r, stroke);
                // Thumb hole
                painter.circle_filled(pos2(center.x + w * 0.16, center.y + h * 0.14), w * 0.07, stroke_color);
                // Color dots
                painter.circle_stroke(pos2(center.x - w * 0.16, center.y - h * 0.14), w * 0.06, stroke);
                painter.circle_stroke(pos2(center.x + w * 0.02, center.y - h * 0.22), w * 0.06, stroke);
                painter.circle_stroke(pos2(center.x - w * 0.20, center.y + h * 0.06), w * 0.06, stroke);
            }

            IconKind::Thumbnails => {
                let frame = Rect::from_min_max(pos2(p.x + w * 0.16, p.y + h * 0.16), pos2(p.x + w * 0.84, p.y + h * 0.84));
                painter.rect_stroke(frame, CornerRadius::same(2), stroke, StrokeKind::Middle);
                // Left thumbnail column
                let split_x = p.x + w * 0.38;
                painter.line_segment([pos2(split_x, frame.min.y), pos2(split_x, frame.max.y)], stroke);
                // Mini page boxes in left column
                painter.rect_stroke(Rect::from_min_max(pos2(frame.min.x + 2.0, frame.min.y + 3.0), pos2(split_x - 2.0, frame.min.y + h * 0.30)), CornerRadius::ZERO, Stroke::new(stroke.width * 0.8, stroke_color), StrokeKind::Middle);
                painter.rect_stroke(Rect::from_min_max(pos2(frame.min.x + 2.0, frame.min.y + h * 0.36), pos2(split_x - 2.0, frame.max.y - 3.0)), CornerRadius::ZERO, Stroke::new(stroke.width * 0.8, stroke_color), StrokeKind::Middle);
            }

            IconKind::Search => {
                let r = w * 0.24;
                let center = pos2(p.x + w * 0.40, p.y + h * 0.40);
                painter.circle_stroke(center, r, stroke);
                let handle_start = pos2(center.x + r * 0.707, center.y + r * 0.707);
                let handle_end = pos2(p.x + w * 0.82, p.y + h * 0.82);
                painter.line_segment([handle_start, handle_end], Stroke::new(stroke.width * 1.3, stroke_color));
            }

            IconKind::Undo => {
                let cx = rect.center().x;
                let cy = rect.center().y;
                painter.line_segment([pos2(cx - w * 0.16, cy - h * 0.18), pos2(cx - w * 0.32, cy)], stroke);
                painter.line_segment([pos2(cx - w * 0.32, cy), pos2(cx - w * 0.16, cy + h * 0.18)], stroke);
                painter.line_segment([pos2(cx - w * 0.32, cy), pos2(cx + w * 0.14, cy)], stroke);
                painter.line_segment([pos2(cx + w * 0.14, cy), pos2(cx + w * 0.24, cy + h * 0.20)], stroke);
            }

            IconKind::Redo => {
                let cx = rect.center().x;
                let cy = rect.center().y;
                painter.line_segment([pos2(cx + w * 0.16, cy - h * 0.18), pos2(cx + w * 0.32, cy)], stroke);
                painter.line_segment([pos2(cx + w * 0.32, cy), pos2(cx + w * 0.16, cy + h * 0.18)], stroke);
                painter.line_segment([pos2(cx + w * 0.32, cy), pos2(cx - w * 0.14, cy)], stroke);
                painter.line_segment([pos2(cx - w * 0.14, cy), pos2(cx - w * 0.24, cy + h * 0.20)], stroke);
            }

            IconKind::ZoomIn => {
                let r = w * 0.24;
                let center = pos2(p.x + w * 0.40, p.y + h * 0.40);
                painter.circle_stroke(center, r, stroke);
                let handle_start = pos2(center.x + r * 0.707, center.y + r * 0.707);
                let handle_end = pos2(p.x + w * 0.82, p.y + h * 0.82);
                painter.line_segment([handle_start, handle_end], Stroke::new(stroke.width * 1.3, stroke_color));
                // Plus inside
                let pr = r * 0.50;
                painter.line_segment([pos2(center.x - pr, center.y), pos2(center.x + pr, center.y)], stroke);
                painter.line_segment([pos2(center.x, center.y - pr), pos2(center.x, center.y + pr)], stroke);
            }

            IconKind::ZoomOut => {
                let r = w * 0.24;
                let center = pos2(p.x + w * 0.40, p.y + h * 0.40);
                painter.circle_stroke(center, r, stroke);
                let handle_start = pos2(center.x + r * 0.707, center.y + r * 0.707);
                let handle_end = pos2(p.x + w * 0.82, p.y + h * 0.82);
                painter.line_segment([handle_start, handle_end], Stroke::new(stroke.width * 1.3, stroke_color));
                // Minus inside
                let pr = r * 0.50;
                painter.line_segment([pos2(center.x - pr, center.y), pos2(center.x + pr, center.y)], stroke);
            }

            IconKind::Close => {
                let left = p.x + w * 0.26;
                let right = p.x + w * 0.74;
                let top = p.y + h * 0.26;
                let bottom = p.y + h * 0.74;
                painter.line_segment([pos2(left, top), pos2(right, bottom)], stroke);
                painter.line_segment([pos2(right, top), pos2(left, bottom)], stroke);
            }

            IconKind::Check => {
                let c1 = pos2(p.x + w * 0.22, p.y + h * 0.52);
                let c2 = pos2(p.x + w * 0.42, p.y + h * 0.74);
                let c3 = pos2(p.x + w * 0.82, p.y + h * 0.28);
                painter.line_segment([c1, c2], stroke);
                painter.line_segment([c2, c3], stroke);
            }

            IconKind::About => {
                Self::draw_logo(painter, rect, stroke_color);
            }

            IconKind::ExportPdf => {
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.76;
                let top = p.y + h * 0.14;
                let bottom = p.y + h * 0.86;
                let fold = w * 0.20;
                let pts = [
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bottom),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                painter.line_segment([pos2(right - fold, top), pos2(right - fold, top + fold)], stroke);
                painter.line_segment([pos2(right - fold, top + fold), pos2(right, top + fold)], stroke);
                // Outward export curved/directed arrow
                let ax = p.x + w * 0.45;
                let ay = p.y + h * 0.52;
                let arx = p.x + w * 0.88;
                painter.line_segment([pos2(ax, ay), pos2(arx, ay)], Stroke::new(stroke.width * 1.2, stroke_color));
                painter.line_segment([pos2(arx, ay), pos2(arx - w * 0.14, ay - h * 0.12)], stroke);
                painter.line_segment([pos2(arx, ay), pos2(arx - w * 0.14, ay + h * 0.12)], stroke);
            }

            IconKind::FillAndSign => {
                let tip = pos2(p.x + w * 0.28, p.y + h * 0.68);
                let top_nib = pos2(p.x + w * 0.72, p.y + h * 0.18);
                painter.line_segment([tip, top_nib], Stroke::new(stroke.width * 1.3, stroke_color));
                painter.line_segment([pos2(p.x + w * 0.24, p.y + h * 0.62), tip], stroke);
                painter.line_segment([pos2(p.x + w * 0.34, p.y + h * 0.72), tip], stroke);
                let s_pts = [
                    pos2(p.x + w * 0.16, p.y + h * 0.84),
                    pos2(p.x + w * 0.38, p.y + h * 0.78),
                    pos2(p.x + w * 0.52, p.y + h * 0.88),
                    pos2(p.x + w * 0.70, p.y + h * 0.82),
                    pos2(p.x + w * 0.86, p.y + h * 0.86),
                ];
                for i in 0..s_pts.len() - 1 {
                    painter.line_segment([s_pts[i], s_pts[i + 1]], stroke);
                }
            }

            IconKind::EditPdf => {
                let left = p.x + w * 0.18;
                let right = p.x + w * 0.72;
                let top = p.y + h * 0.14;
                let bottom = p.y + h * 0.86;
                let fold = w * 0.18;
                let pts = [
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bottom - h * 0.22),
                    pos2(right - w * 0.20, bottom),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                painter.line_segment([pos2(right - fold, top), pos2(right - fold, top + fold)], stroke);
                painter.line_segment([pos2(right - fold, top + fold), pos2(right, top + fold)], stroke);
                let p_tip = pos2(p.x + w * 0.56, p.y + h * 0.78);
                let p_top = pos2(p.x + w * 0.84, p.y + h * 0.48);
                painter.line_segment([p_tip, p_top], Stroke::new(stroke.width * 1.4, stroke_color));
                painter.line_segment([pos2(p_tip.x - w * 0.05, p_tip.y + h * 0.05), p_tip], stroke);
            }

            IconKind::CreatePdf => {
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.80;
                let top = p.y + h * 0.14;
                let bottom = p.y + h * 0.86;
                let fold = w * 0.20;
                let pts = [
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bottom - h * 0.26),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                painter.line_segment([pts[0], pts[1]], stroke);
                painter.line_segment([pts[1], pts[2]], stroke);
                painter.line_segment([pts[2], pts[3]], stroke);
                painter.line_segment([pts[4], pts[5]], stroke);
                painter.line_segment([pos2(right - fold, top), pos2(right - fold, top + fold)], stroke);
                painter.line_segment([pos2(right - fold, top + fold), pos2(right, top + fold)], stroke);
                let cx = p.x + w * 0.68;
                let cy = p.y + h * 0.74;
                let pr = w * 0.16;
                painter.line_segment([pos2(cx - pr, cy), pos2(cx + pr, cy)], Stroke::new(stroke.width * 1.3, stroke_color));
                painter.line_segment([pos2(cx, cy - pr), pos2(cx, cy + pr)], Stroke::new(stroke.width * 1.3, stroke_color));
            }

            IconKind::CombineFiles => {
                let r1 = Rect::from_min_size(pos2(p.x + w * 0.16, p.y + h * 0.24), vec2(w * 0.48, h * 0.60));
                let r2 = Rect::from_min_size(pos2(p.x + w * 0.36, p.y + h * 0.14), vec2(w * 0.48, h * 0.60));
                painter.rect_stroke(r2, CornerRadius::same(2), stroke, StrokeKind::Middle);
                painter.rect_stroke(r1, CornerRadius::same(2), stroke, StrokeKind::Middle);
                painter.line_segment([pos2(r1.min.x + w * 0.08, r1.min.y + h * 0.20), pos2(r1.max.x - w * 0.08, r1.min.y + h * 0.20)], stroke);
                painter.line_segment([pos2(r1.min.x + w * 0.08, r1.min.y + h * 0.38), pos2(r1.max.x - w * 0.14, r1.min.y + h * 0.38)], stroke);
            }

            IconKind::OrganizePages => {
                let gap = w * 0.10;
                let sq_w = (w * 0.70 - gap) * 0.5;
                let sq_h = (h * 0.70 - gap) * 0.5;
                let start_x = p.x + w * 0.15;
                let start_y = p.y + h * 0.15;
                for row in 0..2 {
                    for col in 0..2 {
                        let rx = start_x + (col as f32) * (sq_w + gap);
                        let ry = start_y + (row as f32) * (sq_h + gap);
                        let sq_rect = Rect::from_min_size(pos2(rx, ry), vec2(sq_w, sq_h));
                        painter.rect_stroke(sq_rect, CornerRadius::same(3), stroke, StrokeKind::Middle);
                    }
                }
            }

            IconKind::AddComments => {
                let b_left = p.x + w * 0.15;
                let b_right = p.x + w * 0.85;
                let b_top = p.y + h * 0.18;
                let b_bot = p.y + h * 0.68;
                let bubble_pts = [
                    pos2(b_left + 4.0, b_top),
                    pos2(b_right - 4.0, b_top),
                    pos2(b_right, b_top + 4.0),
                    pos2(b_right, b_bot - 4.0),
                    pos2(b_right - 4.0, b_bot),
                    pos2(b_left + w * 0.35, b_bot),
                    pos2(b_left + w * 0.16, p.y + h * 0.86),
                    pos2(b_left + w * 0.20, b_bot),
                    pos2(b_left + 4.0, b_bot),
                    pos2(b_left, b_bot - 4.0),
                    pos2(b_left, b_top + 4.0),
                    pos2(b_left + 4.0, b_top),
                ];
                for i in 0..bubble_pts.len() - 1 {
                    painter.line_segment([bubble_pts[i], bubble_pts[i + 1]], stroke);
                }
                painter.line_segment([pos2(b_left + w * 0.12, b_top + h * 0.18), pos2(b_right - w * 0.12, b_top + h * 0.18)], stroke);
                painter.line_segment([pos2(b_left + w * 0.12, b_top + h * 0.32), pos2(b_right - w * 0.24, b_top + h * 0.32)], stroke);
            }

            IconKind::ScanOcr => {
                let d = w * 0.16;
                let x0 = p.x + w * 0.14;
                let x1 = p.x + w * 0.86;
                let y0 = p.y + h * 0.14;
                let y1 = p.y + h * 0.86;
                painter.line_segment([pos2(x0, y0 + d), pos2(x0, y0)], stroke);
                painter.line_segment([pos2(x0, y0), pos2(x0 + d, y0)], stroke);
                painter.line_segment([pos2(x1 - d, y0), pos2(x1, y0)], stroke);
                painter.line_segment([pos2(x1, y0), pos2(x1, y0 + d)], stroke);
                painter.line_segment([pos2(x0, y1 - d), pos2(x0, y1)], stroke);
                painter.line_segment([pos2(x0, y1), pos2(x0 + d, y1)], stroke);
                painter.line_segment([pos2(x1 - d, y1), pos2(x1, y1)], stroke);
                painter.line_segment([pos2(x1, y1), pos2(x1, y1 - d)], stroke);
                painter.line_segment([pos2(p.x + w * 0.30, p.y + h * 0.38), pos2(p.x + w * 0.70, p.y + h * 0.38)], stroke);
                painter.line_segment([pos2(p.x + w * 0.30, p.y + h * 0.50), pos2(p.x + w * 0.70, p.y + h * 0.50)], stroke);
                painter.line_segment([pos2(p.x + w * 0.30, p.y + h * 0.62), pos2(p.x + w * 0.58, p.y + h * 0.62)], stroke);
            }

            IconKind::ProtectPdf => {
                let cx = rect.center().x;
                let top = p.y + h * 0.16;
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.80;
                let bot = p.y + h * 0.88;
                let shield_pts = [
                    pos2(left, top),
                    pos2(right, top),
                    pos2(right, p.y + h * 0.54),
                    pos2(cx, bot),
                    pos2(left, p.y + h * 0.54),
                    pos2(left, top),
                ];
                for i in 0..shield_pts.len() - 1 {
                    painter.line_segment([shield_pts[i], shield_pts[i + 1]], stroke);
                }
                let c1 = pos2(cx - w * 0.14, p.y + h * 0.46);
                let c2 = pos2(cx - w * 0.04, p.y + h * 0.56);
                let c3 = pos2(cx + w * 0.15, p.y + h * 0.36);
                painter.line_segment([c1, c2], stroke);
                painter.line_segment([c2, c3], stroke);
            }

            IconKind::RedactPdf => {
                let r = Rect::from_min_max(pos2(p.x + w * 0.16, p.y + h * 0.30), pos2(p.x + w * 0.84, p.y + h * 0.70));
                painter.rect_stroke(r, CornerRadius::same(3), stroke, StrokeKind::Middle);
                painter.rect_filled(r.shrink(2.0), CornerRadius::same(2), stroke_color);
            }

            IconKind::CompressPdf => {
                let left = p.x + w * 0.22;
                let right = p.x + w * 0.78;
                let top = p.y + h * 0.14;
                let bottom = p.y + h * 0.76;
                let fold = w * 0.18;
                let pts = [
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bottom),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                let cx = rect.center().x;
                let ay = p.y + h * 0.32;
                let aby = p.y + h * 0.62;
                painter.line_segment([pos2(cx, ay), pos2(cx, aby)], Stroke::new(stroke.width * 1.2, stroke_color));
                painter.line_segment([pos2(cx, aby), pos2(cx - w * 0.12, aby - h * 0.10)], stroke);
                painter.line_segment([pos2(cx, aby), pos2(cx + w * 0.12, aby - h * 0.10)], stroke);
                painter.line_segment([pos2(left, p.y + h * 0.86), pos2(right, p.y + h * 0.86)], Stroke::new(stroke.width * 1.2, stroke_color));
            }

            IconKind::PrepareForm => {
                let box_rect = Rect::from_min_max(pos2(p.x + w * 0.14, p.y + h * 0.30), pos2(p.x + w * 0.86, p.y + h * 0.70));
                painter.rect_stroke(box_rect, CornerRadius::same(4), stroke, StrokeKind::Middle);
                let cx = p.x + w * 0.35;
                let cy1 = p.y + h * 0.38;
                let cy2 = p.y + h * 0.62;
                painter.line_segment([pos2(cx, cy1), pos2(cx, cy2)], stroke);
                painter.line_segment([pos2(cx - w * 0.06, cy1), pos2(cx + w * 0.06, cy1)], stroke);
                painter.line_segment([pos2(cx - w * 0.06, cy2), pos2(cx + w * 0.06, cy2)], stroke);
                painter.line_segment([pos2(p.x + w * 0.50, p.y + h * 0.50), pos2(p.x + w * 0.74, p.y + h * 0.50)], stroke);
            }

            IconKind::AddStamp => {
                let cx = rect.center().x;
                painter.circle_stroke(pos2(cx, p.y + h * 0.22), w * 0.08, stroke);
                painter.line_segment([pos2(cx, p.y + h * 0.30), pos2(cx, p.y + h * 0.54)], stroke);
                let b_rect = Rect::from_min_max(pos2(p.x + w * 0.20, p.y + h * 0.54), pos2(p.x + w * 0.80, p.y + h * 0.76));
                painter.rect_stroke(b_rect, CornerRadius::same(2), stroke, StrokeKind::Middle);
                painter.line_segment([pos2(p.x + w * 0.16, p.y + h * 0.84), pos2(p.x + w * 0.84, p.y + h * 0.84)], Stroke::new(stroke.width * 1.2, stroke_color));
            }

            IconKind::UseCertificate => {
                let cx = rect.center().x;
                let cy = p.y + h * 0.40;
                let r = w * 0.26;
                painter.circle_stroke(pos2(cx, cy), r, stroke);
                let c1 = pos2(cx - w * 0.10, cy);
                let c2 = pos2(cx - w * 0.02, cy + h * 0.08);
                let c3 = pos2(cx + w * 0.12, cy - h * 0.08);
                painter.line_segment([c1, c2], stroke);
                painter.line_segment([c2, c3], stroke);
                painter.line_segment([pos2(cx - w * 0.10, cy + r - 1.0), pos2(cx - w * 0.18, p.y + h * 0.86)], stroke);
                painter.line_segment([pos2(cx - w * 0.18, p.y + h * 0.86), pos2(cx - w * 0.04, p.y + h * 0.78)], stroke);
                painter.line_segment([pos2(cx + w * 0.10, cy + r - 1.0), pos2(cx + w * 0.18, p.y + h * 0.86)], stroke);
                painter.line_segment([pos2(cx + w * 0.18, p.y + h * 0.86), pos2(cx + w * 0.04, p.y + h * 0.78)], stroke);
            }

            IconKind::PrintProduction => {
                let body = Rect::from_min_max(pos2(p.x + w * 0.18, p.y + h * 0.36), pos2(p.x + w * 0.82, p.y + h * 0.72));
                painter.rect_stroke(body, CornerRadius::same(3), stroke, StrokeKind::Middle);
                painter.rect_stroke(Rect::from_min_max(pos2(p.x + w * 0.30, p.y + h * 0.18), pos2(p.x + w * 0.70, p.y + h * 0.36)), CornerRadius::ZERO, stroke, StrokeKind::Middle);
                painter.line_segment([pos2(p.x + w * 0.28, p.y + h * 0.72), pos2(p.x + w * 0.28, p.y + h * 0.86)], stroke);
                painter.line_segment([pos2(p.x + w * 0.28, p.y + h * 0.86), pos2(p.x + w * 0.72, p.y + h * 0.86)], stroke);
                painter.line_segment([pos2(p.x + w * 0.72, p.y + h * 0.86), pos2(p.x + w * 0.72, p.y + h * 0.72)], stroke);
                painter.circle_filled(pos2(p.x + w * 0.72, p.y + h * 0.46), w * 0.03, stroke_color);
            }

            IconKind::MeasureObjects => {
                let r_start = pos2(p.x + w * 0.20, p.y + h * 0.80);
                let r_end = pos2(p.x + w * 0.80, p.y + h * 0.20);
                painter.line_segment([r_start, r_end], Stroke::new(stroke.width * 1.5, stroke_color));
                for i in 1..5 {
                    let t = (i as f32) / 5.0;
                    let mx = r_start.x + (r_end.x - r_start.x) * t;
                    let my = r_start.y + (r_end.y - r_start.y) * t;
                    let len = if i % 2 == 0 { w * 0.09 } else { w * 0.05 };
                    painter.line_segment([pos2(mx, my), pos2(mx - len * 0.707, my - len * 0.707)], stroke);
                }
            }

            IconKind::CompareFiles => {
                let r1 = Rect::from_min_size(pos2(p.x + w * 0.16, p.y + h * 0.18), vec2(w * 0.32, h * 0.50));
                let r2 = Rect::from_min_size(pos2(p.x + w * 0.52, p.y + h * 0.18), vec2(w * 0.32, h * 0.50));
                painter.rect_stroke(r1, CornerRadius::same(2), stroke, StrokeKind::Middle);
                painter.rect_stroke(r2, CornerRadius::same(2), stroke, StrokeKind::Middle);
                let ay1 = p.y + h * 0.78;
                let ay2 = p.y + h * 0.88;
                painter.line_segment([pos2(p.x + w * 0.24, ay1), pos2(p.x + w * 0.76, ay1)], stroke);
                painter.line_segment([pos2(p.x + w * 0.76, ay1), pos2(p.x + w * 0.68, ay1 - h * 0.06)], stroke);
                painter.line_segment([pos2(p.x + w * 0.76, ay2), pos2(p.x + w * 0.24, ay2)], stroke);
                painter.line_segment([pos2(p.x + w * 0.24, ay2), pos2(p.x + w * 0.32, ay2 + h * 0.06)], stroke);
            }

            IconKind::GuidedActions => {
                let left = p.x + w * 0.18;
                let text_start = p.x + w * 0.42;
                let right = p.x + w * 0.82;
                for i in 0..3 {
                    let y = p.y + h * (0.28 + (i as f32) * 0.24);
                    painter.line_segment([pos2(left, y), pos2(left + w * 0.06, y + h * 0.06)], stroke);
                    painter.line_segment([pos2(left + w * 0.06, y + h * 0.06), pos2(left + w * 0.16, y - h * 0.06)], stroke);
                    painter.line_segment([pos2(text_start, y), pos2(right, y)], stroke);
                }
            }

            IconKind::Accessibility => {
                let cx = rect.center().x;
                let cy = rect.center().y;
                painter.circle_stroke(pos2(cx, cy), w * 0.38, stroke);
                painter.circle_filled(pos2(cx, cy - h * 0.16), w * 0.05, stroke_color);
                painter.line_segment([pos2(cx - w * 0.20, cy - h * 0.04), pos2(cx + w * 0.20, cy - h * 0.04)], stroke);
                painter.line_segment([pos2(cx, cy - h * 0.08), pos2(cx, cy + h * 0.12)], stroke);
                painter.line_segment([pos2(cx, cy + h * 0.12), pos2(cx - w * 0.12, cy + h * 0.28)], stroke);
                painter.line_segment([pos2(cx, cy + h * 0.12), pos2(cx + w * 0.12, cy + h * 0.28)], stroke);
            }

            IconKind::PdfStandards => {
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.80;
                let top = p.y + h * 0.14;
                let bottom = p.y + h * 0.86;
                let fold = w * 0.20;
                let pts = [
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bottom),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                let cx = rect.center().x;
                let cy = rect.center().y + h * 0.06;
                painter.circle_stroke(pos2(cx, cy), w * 0.14, stroke);
                painter.line_segment([pos2(cx - w * 0.06, cy), pos2(cx - w * 0.01, cy + h * 0.05)], stroke);
                painter.line_segment([pos2(cx - w * 0.01, cy + h * 0.05), pos2(cx + w * 0.07, cy - h * 0.04)], stroke);
            }

            IconKind::SearchIndex => {
                let left = p.x + w * 0.18;
                let right = p.x + w * 0.72;
                let top = p.y + h * 0.14;
                let bottom = p.y + h * 0.84;
                let fold = w * 0.18;
                let pts = [
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bottom - h * 0.24),
                    pos2(left, bottom),
                    pos2(left, top),
                ];
                painter.line_segment([pts[0], pts[1]], stroke);
                painter.line_segment([pts[1], pts[2]], stroke);
                painter.line_segment([pts[2], pts[3]], stroke);
                painter.line_segment([pts[4], pts[5]], stroke);
                let mag_c = pos2(p.x + w * 0.65, p.y + h * 0.65);
                let mr = w * 0.14;
                painter.circle_stroke(mag_c, mr, stroke);
                painter.line_segment([pos2(mag_c.x + mr * 0.707, mag_c.y + mr * 0.707), pos2(p.x + w * 0.86, p.y + h * 0.86)], stroke);
            }

            IconKind::JavaScript => {
                let cx = rect.center().x;
                let bx1 = cx - w * 0.24;
                let bx2 = cx - w * 0.12;
                let y1 = p.y + h * 0.20;
                let ymid = p.y + h * 0.50;
                let y2 = p.y + h * 0.80;
                painter.line_segment([pos2(bx2, y1), pos2(bx1, y1 + h * 0.08)], stroke);
                painter.line_segment([pos2(bx1, y1 + h * 0.08), pos2(bx1, ymid - h * 0.06)], stroke);
                painter.line_segment([pos2(bx1, ymid - h * 0.06), pos2(bx1 - w * 0.06, ymid)], stroke);
                painter.line_segment([pos2(bx1 - w * 0.06, ymid), pos2(bx1, ymid + h * 0.06)], stroke);
                painter.line_segment([pos2(bx1, ymid + h * 0.06), pos2(bx1, y2 - h * 0.08)], stroke);
                painter.line_segment([pos2(bx1, y2 - h * 0.08), pos2(bx2, y2)], stroke);
                let rx1 = cx + w * 0.24;
                let rx2 = cx + w * 0.12;
                painter.line_segment([pos2(rx2, y1), pos2(rx1, y1 + h * 0.08)], stroke);
                painter.line_segment([pos2(rx1, y1 + h * 0.08), pos2(rx1, ymid - h * 0.06)], stroke);
                painter.line_segment([pos2(rx1, ymid - h * 0.06), pos2(rx1 + w * 0.06, ymid)], stroke);
                painter.line_segment([pos2(rx1 + w * 0.06, ymid), pos2(rx1, ymid + h * 0.06)], stroke);
                painter.line_segment([pos2(rx1, ymid + h * 0.06), pos2(rx1, y2 - h * 0.08)], stroke);
                painter.line_segment([pos2(rx1, y2 - h * 0.08), pos2(rx2, y2)], stroke);
            }

            IconKind::FolderOpen => {
                let left = p.x + w * 0.16;
                let right = p.x + w * 0.84;
                let top = p.y + h * 0.28;
                let bottom = p.y + h * 0.78;
                painter.line_segment([pos2(left, top), pos2(left + w * 0.28, top)], stroke);
                painter.line_segment([pos2(left + w * 0.28, top), pos2(left + w * 0.36, top - h * 0.08)], stroke);
                painter.line_segment([pos2(left + w * 0.36, top - h * 0.08), pos2(left + w * 0.60, top - h * 0.08)], stroke);
                painter.line_segment([pos2(left + w * 0.60, top - h * 0.08), pos2(left + w * 0.66, top)], stroke);
                painter.line_segment([pos2(left + w * 0.66, top), pos2(right, top)], stroke);
                painter.line_segment([pos2(right, top), pos2(right - w * 0.06, bottom)], stroke);
                painter.line_segment([pos2(right - w * 0.06, bottom), pos2(left + w * 0.06, bottom)], stroke);
                painter.line_segment([pos2(left + w * 0.06, bottom), pos2(left, top)], stroke);
            }

            IconKind::AddText => {
                let left = p.x + w * 0.22;
                let right = p.x + w * 0.78;
                let top = p.y + h * 0.20;
                let bot = p.y + h * 0.80;
                let cx = (left + right) * 0.5;
                painter.line_segment([pos2(left, top), pos2(right, top)], Stroke::new(stroke.width * 1.3, stroke_color));
                painter.line_segment([pos2(left, top), pos2(left, top + h * 0.08)], stroke);
                painter.line_segment([pos2(right, top), pos2(right, top + h * 0.08)], stroke);
                painter.line_segment([pos2(cx, top), pos2(cx, bot)], Stroke::new(stroke.width * 1.3, stroke_color));
                painter.line_segment([pos2(cx - w * 0.12, bot), pos2(cx + w * 0.12, bot)], stroke);
            }

            IconKind::AddImage => {
                let frame = Rect::from_min_max(pos2(p.x + w * 0.16, p.y + h * 0.18), pos2(p.x + w * 0.84, p.y + h * 0.82));
                painter.rect_stroke(frame, CornerRadius::same(3), stroke, StrokeKind::Middle);
                painter.circle_stroke(pos2(frame.min.x + w * 0.18, frame.min.y + h * 0.18), w * 0.06, stroke);
                let m1 = pos2(frame.min.x, frame.max.y - h * 0.12);
                let m2 = pos2(frame.min.x + w * 0.26, frame.min.y + h * 0.36);
                let m3 = pos2(frame.min.x + w * 0.44, frame.max.y - h * 0.08);
                let m4 = pos2(frame.min.x + w * 0.56, frame.min.y + h * 0.46);
                let m5 = pos2(frame.max.x, frame.max.y - h * 0.06);
                painter.line_segment([m1, m2], stroke);
                painter.line_segment([m2, m3], stroke);
                painter.line_segment([m3, m4], stroke);
                painter.line_segment([m4, m5], stroke);
            }

            IconKind::HeaderFooter => {
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.80;
                let top = p.y + h * 0.18;
                let bot = p.y + h * 0.82;
                painter.line_segment([pos2(left, top), pos2(left, bot)], stroke);
                painter.line_segment([pos2(right, top), pos2(right, bot)], stroke);
                painter.line_segment([pos2(left, top + h * 0.12), pos2(right, top + h * 0.12)], Stroke::new(stroke.width * 1.3, stroke_color));
                painter.line_segment([pos2(left, bot - h * 0.12), pos2(right, bot - h * 0.12)], Stroke::new(stroke.width * 1.3, stroke_color));
            }

            IconKind::Watermark => {
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.80;
                let top = p.y + h * 0.20;
                let bot = p.y + h * 0.80;
                painter.rect_stroke(Rect::from_min_max(pos2(left, top), pos2(right, bot)), CornerRadius::same(3), stroke, StrokeKind::Middle);
                let cx = rect.center().x;
                painter.circle_stroke(pos2(cx, top + h * 0.18), w * 0.07, stroke);
                painter.line_segment([pos2(cx, top + h * 0.25), pos2(cx, bot - h * 0.18)], stroke);
                painter.line_segment([pos2(cx - w * 0.18, bot - h * 0.18), pos2(cx + w * 0.18, bot - h * 0.18)], Stroke::new(stroke.width * 1.3, stroke_color));
            }

            IconKind::Link => {
                let cx = rect.center().x;
                let cy = rect.center().y;
                let d = w * 0.14;
                let r1 = Rect::from_center_size(pos2(cx - d * 0.6, cy - d * 0.6), vec2(w * 0.36, h * 0.18));
                let r2 = Rect::from_center_size(pos2(cx + d * 0.6, cy + d * 0.6), vec2(w * 0.36, h * 0.18));
                painter.rect_stroke(r1, CornerRadius::same(4), stroke, StrokeKind::Middle);
                painter.rect_stroke(r2, CornerRadius::same(4), stroke, StrokeKind::Middle);
            }

            IconKind::BatesNumbering => {
                let left = p.x + w * 0.25;
                let right = p.x + w * 0.75;
                let top = p.y + h * 0.20;
                let bot = p.y + h * 0.80;
                let v1 = p.x + w * 0.40;
                let v2 = p.x + w * 0.60;
                painter.line_segment([pos2(v1 + 2.0, top), pos2(v1 - 2.0, bot)], stroke);
                painter.line_segment([pos2(v2 + 2.0, top), pos2(v2 - 2.0, bot)], stroke);
                let h1 = p.y + h * 0.40;
                let h2 = p.y + h * 0.60;
                painter.line_segment([pos2(left, h1), pos2(right, h1)], stroke);
                painter.line_segment([pos2(left, h2), pos2(right, h2)], stroke);
            }

            IconKind::Spreadsheet => {
                let frame = Rect::from_min_max(pos2(p.x + w * 0.16, p.y + h * 0.18), pos2(p.x + w * 0.84, p.y + h * 0.82));
                painter.rect_stroke(frame, CornerRadius::same(2), stroke, StrokeKind::Middle);
                let mid_x = frame.center().x;
                let y1 = frame.min.y + frame.height() * 0.33;
                let y2 = frame.min.y + frame.height() * 0.66;
                painter.line_segment([pos2(mid_x, frame.min.y), pos2(mid_x, frame.max.y)], stroke);
                painter.line_segment([pos2(frame.min.x, y1), pos2(frame.max.x, y1)], stroke);
                painter.line_segment([pos2(frame.min.x, y2), pos2(frame.max.x, y2)], stroke);
            }

            IconKind::Presentation => {
                let frame = Rect::from_min_max(pos2(p.x + w * 0.16, p.y + h * 0.18), pos2(p.x + w * 0.84, p.y + h * 0.64));
                painter.rect_stroke(frame, CornerRadius::same(2), stroke, StrokeKind::Middle);
                let cx = frame.center().x;
                let sy = frame.max.y;
                painter.line_segment([pos2(cx, sy), pos2(cx, sy + h * 0.10)], stroke);
                painter.line_segment([pos2(cx, sy + h * 0.10), pos2(cx - w * 0.16, sy + h * 0.22)], stroke);
                painter.line_segment([pos2(cx, sy + h * 0.10), pos2(cx + w * 0.16, sy + h * 0.22)], stroke);
            }

            IconKind::Dot => {
                let center = rect.center();
                painter.circle_stroke(center, w * 0.28, stroke);
                painter.circle_filled(center, stroke.width * 1.4, stroke_color);
            }

            IconKind::Line => {
                let cy = rect.center().y;
                painter.line_segment(
                    [pos2(p.x + w * 0.20, cy), pos2(p.x + w * 0.80, cy)],
                    Stroke::new(stroke.width * 1.5, stroke_color),
                );
            }

            IconKind::Date => {
                let center = rect.center();
                let r = w * 0.32;
                painter.circle_stroke(center, r, stroke);
                painter.line_segment([center, pos2(center.x, center.y - r * 0.65)], stroke);
                painter.line_segment([center, pos2(center.x + r * 0.50, center.y)], stroke);
            }

            IconKind::ExtractPages => {
                let left = p.x + w * 0.20;
                let right = p.x + w * 0.72;
                let top = p.y + h * 0.16;
                let bot = p.y + h * 0.84;
                let fold = w * 0.18;
                let pts = [
                    pos2(left, top),
                    pos2(right - fold, top),
                    pos2(right, top + fold),
                    pos2(right, bot),
                    pos2(left, bot),
                    pos2(left, top),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                let ax = p.x + w * 0.50;
                let ay = p.y + h * 0.50;
                let arx = p.x + w * 0.86;
                painter.line_segment([pos2(ax, ay), pos2(arx, ay)], Stroke::new(stroke.width * 1.2, stroke_color));
                painter.line_segment([pos2(arx, ay), pos2(arx - w * 0.12, ay - h * 0.10)], stroke);
                painter.line_segment([pos2(arx, ay), pos2(arx - w * 0.12, ay + h * 0.10)], stroke);
            }
        }
    }

    /// Helper to render an allocated vector icon into a Ui.
    pub fn render_icon(ui: &mut Ui, kind: IconKind, size: egui::Vec2, color: Color32) {
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        Self::draw_icon(ui.painter(), rect, kind, color);
    }

    /// Loads the raster PNG app logo as an `egui::IconData` for window decoration.
    pub fn load_app_icon() -> Option<egui::IconData> {
        if let Ok(img) = image::load_from_memory(Self::LOGO_PNG) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            })
        } else {
            None
        }
    }
}
