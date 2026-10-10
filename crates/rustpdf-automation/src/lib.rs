//! rustpdf-automation — Typed headless automation interface and script runner.

use rustpdf_engine::DocumentSession;
use rustpdf_organize::SplitRule;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AutomationError {
    #[error("engine error: {0}")]
    Engine(#[from] rustpdf_engine::EngineError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("execution error: {0}")]
    Execution(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum AutomationCommand {
    Inspect {
        path: PathBuf,
    },
    GetPageCount {
        path: PathBuf,
    },
    RotatePages {
        path: PathBuf,
        pages: Vec<usize>,
        degrees: i32,
        output_path: PathBuf,
    },
    Combine {
        input_paths: Vec<PathBuf>,
        output_path: PathBuf,
    },
    Split {
        path: PathBuf,
        every_n_pages: usize,
        output_dir: PathBuf,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutomationResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub struct AutomationRunner;

impl AutomationRunner {
    pub fn execute(cmd: AutomationCommand) -> Result<AutomationResponse, AutomationError> {
        match cmd {
            AutomationCommand::Inspect { path } => {
                let session = DocumentSession::open_file(&path)?;
                let data = serde_json::to_value(&session.model)
                    .map_err(|e| AutomationError::Execution(e.to_string()))?;
                Ok(AutomationResponse {
                    success: true,
                    message: format!("Inspected {} successfully", path.display()),
                    data: Some(data),
                })
            }
            AutomationCommand::GetPageCount { path } => {
                let session = DocumentSession::open_file(&path)?;
                let count = session.page_count();
                Ok(AutomationResponse {
                    success: true,
                    message: format!("Document has {count} pages"),
                    data: Some(serde_json::json!({ "page_count": count })),
                })
            }
            AutomationCommand::RotatePages {
                path,
                pages,
                degrees,
                output_path,
            } => {
                let mut session = DocumentSession::open_file(&path)?;
                session.rotate_pages(&pages, degrees)?;
                session.save_to(&output_path)?;
                Ok(AutomationResponse {
                    success: true,
                    message: format!("Rotated pages and saved to {}", output_path.display()),
                    data: None,
                })
            }
            AutomationCommand::Combine {
                input_paths,
                output_path,
            } => {
                let mut buffers = Vec::with_capacity(input_paths.len());
                for p in &input_paths {
                    buffers.push(fs::read(p)?);
                }
                let combined_bytes = rustpdf_organize::combine_documents(&buffers)
                    .map_err(|e| AutomationError::Execution(e.to_string()))?;
                rustpdf_edit::save_atomic(&output_path, &combined_bytes)
                    .map_err(|e| AutomationError::Execution(e.to_string()))?;
                Ok(AutomationResponse {
                    success: true,
                    message: format!("Combined {} documents into {}", input_paths.len(), output_path.display()),
                    data: None,
                })
            }
            AutomationCommand::Split {
                path,
                every_n_pages,
                output_dir,
            } => {
                let bytes = fs::read(&path)?;
                let chunks = rustpdf_organize::split_document(
                    &bytes,
                    SplitRule::EveryN(every_n_pages),
                )
                .map_err(|e| AutomationError::Execution(e.to_string()))?;

                if !output_dir.exists() {
                    fs::create_dir_all(&output_dir)?;
                }

                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("document");

                let mut saved_paths = Vec::new();
                for (i, chunk_bytes) in chunks.iter().enumerate() {
                    let out_file = output_dir.join(format!("{stem}_part_{:03}.pdf", i + 1));
                    rustpdf_edit::save_atomic(&out_file, chunk_bytes)
                        .map_err(|e| AutomationError::Execution(e.to_string()))?;
                    saved_paths.push(out_file.to_string_lossy().to_string());
                }

                Ok(AutomationResponse {
                    success: true,
                    message: format!("Split into {} files", chunks.len()),
                    data: Some(serde_json::json!({ "output_files": saved_paths })),
                })
            }
        }
    }
}
