//! Background page rasterization and worker thread pool.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{render, PixmapSettings, RenderCache, RenderSettings};

pub const MAX_SIDE: f32 = 8192.0;
pub const MAX_PIXELS: f32 = 64.0e6;

/// Clamps requested scale to avoid excessive allocation.
pub fn effective_scale(width_pt: f32, height_pt: f32, scale: f32) -> f32 {
    let (w, h) = (width_pt.max(1.0), height_pt.max(1.0));
    let by_side = MAX_SIDE / w.max(h);
    let by_area = (MAX_PIXELS / (w * h)).sqrt();
    let capped = scale.min(by_side).min(by_area);
    if capped.is_finite() && capped > 0.0 {
        capped
    } else {
        1.0
    }
}

pub fn device_pixels(pt: f32, scale: f32) -> u32 {
    let px = pt * scale;
    if px.is_finite() && px > 0.0 {
        px.ceil().max(1.0) as u32
    } else {
        1
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderRequest {
    pub page_index: usize,
    pub scale: f32,
    pub tag: u64,
}

#[derive(Debug)]
pub struct RenderedPage {
    pub page_index: usize,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub tag: u64,
    pub error: Option<String>,
    pub duration_ms: u32,
}

#[allow(dead_code)]
struct RenderTask {
    pdf_bytes: Arc<Vec<u8>>,
    password: Option<String>,
    req: RenderRequest,
    sender: Sender<RenderedPage>,
}

pub struct RenderPool {
    worker_tx: Sender<RenderTask>,
    result_rx: Receiver<RenderedPage>,
    result_tx: Sender<RenderedPage>,
    #[allow(dead_code)]
    workers: Vec<JoinHandle<()>>,
}

impl RenderPool {
    pub fn new(num_workers: usize) -> Self {
        let (task_tx, task_rx) = channel::<RenderTask>();
        let task_rx = Arc::new(Mutex::new(task_rx));
        let (result_tx, result_rx) = channel::<RenderedPage>();

        let mut workers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers.max(1) {
            let rx = Arc::clone(&task_rx);
            let handle = thread::spawn(move || {
                loop {
                    let task = {
                        let Ok(guard) = rx.lock() else { break };
                        match guard.recv() {
                            Ok(t) => t,
                            Err(_) => break, // Pool shutdown
                        }
                    };

                    let start = std::time::Instant::now();
                    let page_idx = task.req.page_index;
                    let tag = task.req.tag;

                    let res = catch_unwind(AssertUnwindSafe(|| {
                        let pdf = match Pdf::new(Arc::clone(&task.pdf_bytes)) {
                            Ok(p) => p,
                            Err(e) => {
                                return Err(format!("Failed to parse PDF: {e:?}"));
                            }
                        };

                        let pages = pdf.pages();
                        let page = pages
                            .get(page_idx)
                            .ok_or_else(|| format!("Page {page_idx} not found"))?;

                        let (w_pt, h_pt) = page.render_dimensions();
                        if w_pt < 0.5 || h_pt < 0.5 {
                            return Err(format!(
                                "Page {page_idx} has empty bounds ({w_pt}x{h_pt})"
                            ));
                        }

                        let scale = effective_scale(w_pt, h_pt, task.req.scale);

                        let ps = PixmapSettings {
                            x_scale: scale,
                            y_scale: scale,
                            bg_color: WHITE,
                        };

                        let rs = RenderSettings::default();
                        let cache = RenderCache::default();
                        let settings = InterpreterSettings::default();
                        let pixmap = render(page, &cache, &settings, &rs, &ps);

                        let width = pixmap.width() as u32;
                        let height = pixmap.height() as u32;

                        let rgba: Vec<u8> = pixmap.data_as_u8_slice().to_vec();

                        Ok((width, height, rgba))
                    }));

                    let duration_ms = start.elapsed().as_millis() as u32;

                    let rendered = match res {
                        Ok(Ok((w, h, rgba))) => RenderedPage {
                            page_index: page_idx,
                            width: w,
                            height: h,
                            rgba,
                            tag,
                            error: None,
                            duration_ms,
                        },
                        Ok(Err(e)) => RenderedPage {
                            page_index: page_idx,
                            width: 0,
                            height: 0,
                            rgba: Vec::new(),
                            tag,
                            error: Some(e),
                            duration_ms,
                        },
                        Err(_) => RenderedPage {
                            page_index: page_idx,
                            width: 0,
                            height: 0,
                            rgba: Vec::new(),
                            tag,
                            error: Some("Renderer thread panicked on page".to_string()),
                            duration_ms,
                        },
                    };

                    let _ = task.sender.send(rendered);
                }
            });
            workers.push(handle);
        }

        Self {
            worker_tx: task_tx,
            result_rx,
            result_tx,
            workers,
        }
    }

    pub fn submit(&self, pdf_bytes: Arc<Vec<u8>>, password: Option<String>, req: RenderRequest) {
        let _ = self.worker_tx.send(RenderTask {
            pdf_bytes,
            password,
            req,
            sender: self.result_tx.clone(),
        });
    }

    pub fn try_recv(&self) -> Option<RenderedPage> {
        self.result_rx.try_recv().ok()
    }
}

/// Synchronously renders a single page of a PDF document into raw RGBA pixels.
pub fn render_page_sync(
    pdf_bytes: &[u8],
    page_index: usize,
    requested_scale: f32,
) -> Result<(u32, u32, Vec<u8>), String> {
    let pdf = Pdf::new(Arc::new(pdf_bytes.to_vec()))
        .map_err(|e| format!("Failed to parse PDF: {e:?}"))?;
    let pages = pdf.pages();
    let page = pages
        .get(page_index)
        .ok_or_else(|| format!("Page {page_index} not found"))?;

    let (w_pt, h_pt) = page.render_dimensions();
    if w_pt < 0.5 || h_pt < 0.5 {
        return Err(format!(
            "Page {page_index} has empty bounds ({w_pt}x{h_pt})"
        ));
    }

    let scale = effective_scale(w_pt, h_pt, requested_scale);

    let ps = PixmapSettings {
        x_scale: scale,
        y_scale: scale,
        bg_color: WHITE,
    };

    let rs = RenderSettings::default();
    let cache = RenderCache::default();
    let settings = InterpreterSettings::default();
    let pixmap = render(page, &cache, &settings, &rs, &ps);

    let width = pixmap.width() as u32;
    let height = pixmap.height() as u32;
    let rgba = pixmap.data_as_u8_slice().to_vec();

    Ok((width, height, rgba))
}
