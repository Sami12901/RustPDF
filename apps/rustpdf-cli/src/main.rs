//! rustpdf-cli — Command line PDF workbench tool.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rustpdf_engine::DocumentSession;
use rustpdf_organize::SplitRule;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rustpdf-cli",
    version,
    about = "RustPdf Command-Line PDF Workbench"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Display PDF metadata, page count, and structure
    Info {
        /// Input PDF file
        input: PathBuf,
    },
    /// Extract plain text from PDF
    Text {
        /// Input PDF file
        input: PathBuf,
        /// Optional 1-based page number to extract
        #[arg(short, long)]
        page: Option<usize>,
    },
    /// Render a PDF page to a PNG image
    Render {
        /// Input PDF file
        input: PathBuf,
        /// 1-based page number to render
        #[arg(short, long, default_value_t = 1)]
        page: usize,
        /// Output PNG path
        #[arg(short, long)]
        out: PathBuf,
        /// DPI scale multiplier (1.0 = 72 dpi, 2.0 = 144 dpi)
        #[arg(short, long, default_value_t = 2.0)]
        scale: f32,
    },
    /// Combine multiple PDFs into a single file
    Combine {
        /// Input PDF files to merge in order
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        /// Output merged PDF path
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Extract selected pages into a new PDF
    Extract {
        /// Input PDF file
        input: PathBuf,
        /// Comma-separated 1-based pages (e.g. 1,3,5)
        #[arg(short, long, value_delimiter = ',')]
        pages: Vec<usize>,
        /// Output PDF path
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Split PDF document into smaller parts
    Split {
        /// Input PDF file
        input: PathBuf,
        /// Chunk size in pages
        #[arg(short, long, default_value_t = 1)]
        every: usize,
        /// Output directory for split parts
        #[arg(short, long)]
        out_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { input } => {
            let session = DocumentSession::open_file(&input)
                .with_context(|| format!("Failed to open '{}'", input.display()))?;

            println!("=== Document Info: {} ===", input.display());
            println!("Pages:       {}", session.page_count());
            println!("Version:     PDF-{}", session.model.metadata.pdf_version);
            println!("Encrypted:   {}", session.model.metadata.encrypted);
            if let Some(ref title) = session.model.metadata.title {
                println!("Title:       {}", title);
            }
            if let Some(ref author) = session.model.metadata.author {
                println!("Author:      {}", author);
            }
            if let Some(ref subject) = session.model.metadata.subject {
                println!("Subject:     {}", subject);
            }
            if let Some(ref creator) = session.model.metadata.creator {
                println!("Creator:     {}", creator);
            }
            if let Some(ref producer) = session.model.metadata.producer {
                println!("Producer:    {}", producer);
            }
            println!(
                "File size:   {} bytes",
                session.model.metadata.file_size_bytes
            );
            println!("Bookmarks:   {} top-level", session.model.outlines.len());
        }

        Commands::Text { input, page } => {
            let session = DocumentSession::open_file(&input)
                .with_context(|| format!("Failed to open '{}'", input.display()))?;

            let count = session.page_count();
            let page_indices: Vec<usize> = if let Some(p) = page {
                if p == 0 || p > count {
                    anyhow::bail!("Page {} out of range (1..{})", p, count);
                }
                vec![p - 1]
            } else {
                (0..count).collect()
            };

            for idx in page_indices {
                if page.is_none() && idx > 0 {
                    println!("\n--- Page {} ---\n", idx + 1);
                }
                if let Some(p) = session.model.pages.get(idx) {
                    println!("[Page {} ({} x {} pt)]", idx + 1, p.width, p.height);
                }

                let text = session
                    .cos_doc
                    .inner()
                    .extract_text(&[(idx + 1) as u32])
                    .with_context(|| format!("Failed to extract text from page {}", idx + 1))?;
                println!("{text}");
            }
        }

        Commands::Render {
            input,
            page,
            out,
            scale,
        } => {
            let bytes = fs::read(&input)
                .with_context(|| format!("Failed to read '{}'", input.display()))?;
            if page == 0 {
                anyhow::bail!("Page numbers start at 1");
            }
            if !scale.is_finite() || scale <= 0.0 {
                anyhow::bail!("Scale must be a finite number greater than zero");
            }
            let parsed = lopdf::Document::load_mem(&bytes)
                .with_context(|| format!("Failed to parse '{}'", input.display()))?;
            let page_count = parsed.get_pages().len();
            if page > page_count {
                anyhow::bail!("Page {} out of range (1..{})", page, page_count);
            }

            let pool = rustpdf_render::RenderPool::new(1);
            let req = rustpdf_render::RenderRequest {
                page_index: page.saturating_sub(1),
                scale,
                tag: 1,
            };

            pool.submit(std::sync::Arc::new(bytes), None, req);

            let mut rendered = None;
            for _ in 0..100 {
                if let Some(r) = pool.try_recv() {
                    rendered = Some(r);
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            let page_data = rendered.context("Page render timed out")?;
            if let Some(err) = page_data.error {
                anyhow::bail!("Render failed: {err}");
            }

            let img = image::RgbaImage::from_raw(page_data.width, page_data.height, page_data.rgba)
                .context("Failed to construct RGBA image buffer")?;

            if let Some(parent) = out.parent().filter(|path| !path.as_os_str().is_empty()) {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            img.save_with_format(&out, image::ImageFormat::Png)
                .with_context(|| format!("Failed to save image to '{}'", out.display()))?;

            println!(
                "Rendered page {} to '{}' ({}x{} px)",
                page,
                out.display(),
                page_data.width,
                page_data.height
            );
        }

        Commands::Combine { inputs, out } => {
            let mut buffers = Vec::with_capacity(inputs.len());
            for p in &inputs {
                let b = fs::read(p).with_context(|| format!("Failed to read '{}'", p.display()))?;
                buffers.push(b);
            }

            let merged = rustpdf_organize::combine_documents(&buffers)
                .map_err(|e| anyhow::anyhow!("Combine failed: {e}"))?;

            rustpdf_edit::save_atomic(&out, &merged)
                .with_context(|| format!("Failed to save merged PDF to '{}'", out.display()))?;

            println!("Combined {} files into '{}'", inputs.len(), out.display());
        }

        Commands::Extract { input, pages, out } => {
            let bytes = fs::read(&input)
                .with_context(|| format!("Failed to read '{}'", input.display()))?;
            let doc = lopdf::Document::load_mem(&bytes)
                .with_context(|| format!("Failed to parse '{}'", input.display()))?;
            let total = doc.get_pages().len();

            if pages.is_empty() {
                anyhow::bail!("Select at least one page to extract");
            }

            let mut seen = std::collections::HashSet::with_capacity(pages.len());
            let mut selected = Vec::with_capacity(pages.len());
            for page in pages {
                if page == 0 || page > total {
                    anyhow::bail!("Page {} out of range (1..{})", page, total);
                }
                if !seen.insert(page) {
                    anyhow::bail!("Page {} was selected more than once", page);
                }
                selected.push(page - 1);
            }

            let extracted = rustpdf_organize::extract_pages(&bytes, &selected)
                .with_context(|| format!("Failed to extract pages from '{}'", input.display()))?;
            rustpdf_edit::save_atomic(&out, &extracted)
                .with_context(|| format!("Failed to save extracted PDF to '{}'", out.display()))?;
            println!("Extracted {} page(s) into '{}'", selected.len(), out.display());
        }

        Commands::Split {
            input,
            every,
            out_dir,
        } => {
            let bytes = fs::read(&input)?;
            let chunks = rustpdf_organize::split_document(&bytes, SplitRule::EveryN(every))
                .map_err(|e| anyhow::anyhow!("Split failed: {e}"))?;

            if !out_dir.exists() {
                fs::create_dir_all(&out_dir)?;
            }

            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");

            for (i, chunk) in chunks.iter().enumerate() {
                let part_path = out_dir.join(format!("{stem}_part_{:03}.pdf", i + 1));
                rustpdf_edit::save_atomic(&part_path, chunk)?;
                println!("Created '{}'", part_path.display());
            }

            println!(
                "Successfully split into {} files in '{}'",
                chunks.len(),
                out_dir.display()
            );
        }
    }

    Ok(())
}
