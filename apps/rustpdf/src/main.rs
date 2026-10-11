use rustpdf_ui::{icons::Icons, RustPdfApp};
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 840.0])
        .with_min_inner_size([720.0, 480.0])
        .with_title("RustPdf — PDF Workbench");

    if let Some(icon) = Icons::load_app_icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let initial_file = std::env::args().nth(1).map(std::path::PathBuf::from);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "RustPdf — PDF Workbench",
        options,
        Box::new(move |cc| {
            Ok(Box::new(RustPdfApp::new_with_initial_file(
                cc,
                initial_file,
            )))
        }),
    )
}
