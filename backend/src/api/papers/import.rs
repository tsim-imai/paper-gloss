use crate::api::error::AppError;
use crate::models::{Paper, Chunk};
use crate::services::pdf::{PdfExtractor, TextChunker};
use crate::services::arxiv::ArxivDownloader;
use crate::services::latex::LatexChunker;
use axum::{
    extract::{Multipart, State},
    http::{header, HeaderMap, StatusCode},
    Json,
};
use regex::Regex;
use serde::Serialize;
use sqlx::SqlitePool;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub paper_id: String,
    pub status: String,
    pub message: String,
}

/// POST /papers/import - Import paper from file or arXiv URL
/// Supports both file upload and arXiv URL (data-model.md § URL Import Specification)
pub async fn import_paper(
    State(pool): State<SqlitePool>,
    mut multipart: Multipart,
) -> Result<(StatusCode, HeaderMap, Json<ImportResponse>), AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut url: Option<String> = None;
    let mut title: Option<String> = None;

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to parse multipart data: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                let data = field.bytes().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read file data: {}", e))
                })?;
                file_data = Some(data.to_vec());
            }
            "url" => {
                let value = field.text().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read URL: {}", e))
                })?;
                url = Some(value);
            }
            "title" => {
                let value = field.text().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read title: {}", e))
                })?;
                title = Some(value);
            }
            _ => {}
        }
    }

    // Validate: must have either file or URL
    if file_data.is_none() && url.is_none() {
        return Err(AppError::BadRequest(
            "Must provide either 'file' or 'url'".to_string(),
        ));
    }

    // Handle file upload
    if let Some(data) = file_data {
        return handle_file_upload(&pool, data, title).await;
    }

    // Handle URL import (arXiv-only)
    if let Some(url_str) = url {
        let title = title.ok_or_else(|| {
            AppError::BadRequest("Title is required when importing from URL".to_string())
        })?;

        return handle_arxiv_import(&pool, url_str, title).await;
    }

    Err(AppError::InternalServerError(
        "Unexpected import state".to_string(),
    ))
}

/// Handle file upload import
async fn handle_file_upload(
    pool: &SqlitePool,
    file_data: Vec<u8>,
    title: Option<String>,
) -> Result<(StatusCode, HeaderMap, Json<ImportResponse>), AppError> {
    // Check for empty file (FR-005)
    if file_data.is_empty() {
        return Err(AppError::UnprocessableEntity(
            "File is empty (0 bytes)".to_string(),
        ));
    }

    // Validate PDF size
    if file_data.len() > 100 * 1024 * 1024 {
        return Err(AppError::UnprocessableEntity(
            "PDF file too large (max 100 MB)".to_string(),
        ));
    }

    // Validate PDF format (check PDF magic bytes - FR-005)
    if file_data.len() < 5 || &file_data[0..5] != b"%PDF-" {
        return Err(AppError::UnprocessableEntity(
            "Invalid file format. Only PDF files are accepted.".to_string(),
        ));
    }

    // Derive title from filename or use provided
    let title = title.unwrap_or_else(|| "Untitled Paper".to_string());

    // Generate paper ID and file path
    let paper_id = Uuid::new_v4().to_string();
    let file_path = format!("artifacts/papers/{}/source.pdf", paper_id);

    // Create directory
    fs::create_dir_all(format!("artifacts/papers/{}", paper_id))
        .map_err(|e| AppError::InternalServerError(format!("Failed to create directory: {}", e)))?;

    // Write PDF file
    fs::write(&file_path, &file_data)
        .map_err(|e| AppError::InternalServerError(format!("Failed to write PDF: {}", e)))?;

    info!("Saved PDF to {}", file_path);

    // Pre-validate PDF extraction capability (FR-005, FR-007)
    use crate::services::pdf::{PdfExtractor, TextChunker};
    use std::path::Path;
    if let Ok(valid) = PdfExtractor::validate_pdf(Path::new(&file_path)) {
        if !valid {
            warn!("PDF validation indicates potential extraction issues for {}", file_path);
            // Continue anyway - let the processor handle partial extraction
        }
    }

    // Create paper record
    let paper = Paper::create(pool, title.clone(), None, file_path.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create paper in database: {:?}", e);
            AppError::InternalServerError(format!("Failed to create paper: {}", e))
        })?;

    // Extract + chunk synchronously at import time (no auto-translation)
    match PdfExtractor::extract_with_recovery(Path::new(&file_path)) {
        Ok(result) => {
            if !result.text.trim().is_empty() {
                let chunker = TextChunker::default();
                let chunks = chunker.chunk(&result.text);
                for ch in chunks {
                    let _ = Chunk::create(
                        pool,
                        paper.id.clone(),
                        ch.index as i32,
                        ch.text,
                        ch.content_hash,
                        ch.token_count.map(|t| t as i32),
                    ).await;
                }
                info!("Prepared chunks at import for paper {}", paper.id);
            } else {
                warn!("Extraction returned empty text at import for paper {}", paper.id);
            }
        }
        Err(e) => warn!("Extraction failed at import for {}: {}", paper.id, e),
    }

    // Build Location header
    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        format!("/api/papers/{}", paper.id)
            .parse()
            .expect("Invalid header value"),
    );

    Ok((
        StatusCode::CREATED,
        headers,
        Json(ImportResponse {
            paper_id: paper.id,
            status: "pending".to_string(),
            message: "Paper imported successfully. Chunks prepared if possible. Next: POST /api/papers/{id}/translate, then optionally /extract-terms-jp and /scan-jp.".to_string(),
        }),
    ))
}

/// Handle arXiv URL import - downloads LaTeX source and processes
async fn handle_arxiv_import(
    pool: &SqlitePool,
    url: String,
    title: String,
) -> Result<(StatusCode, HeaderMap, Json<ImportResponse>), AppError> {
    // Validate arXiv URL format
    let arxiv_regex = Regex::new(r"^https://(www\.)?arxiv\.org/abs/\d{4}\.\d{4,5}(v\d+)?$")
        .expect("Invalid regex");

    if !arxiv_regex.is_match(&url) {
        return Err(AppError::BadRequest(
            "Only arXiv URLs are supported (https://arxiv.org/abs/...)".to_string(),
        ));
    }

    // Extract arXiv ID from URL
    let arxiv_id = ArxivDownloader::extract_arxiv_id(&url)
        .map_err(|e| AppError::BadRequest(format!("Invalid arXiv URL: {}", e)))?;

    info!("Importing arXiv paper {} ({})", arxiv_id, title);

    // Generate paper ID and create directory
    let paper_id = Uuid::new_v4().to_string();
    let paper_dir = PathBuf::from(format!("artifacts/papers/{}", paper_id));

    fs::create_dir_all(&paper_dir)
        .map_err(|e| AppError::InternalServerError(format!("Failed to create directory: {}", e)))?;

    // Download and extract arXiv source
    let downloader = ArxivDownloader::new(paper_dir.clone())
        .map_err(|e| AppError::InternalServerError(format!("Failed to create downloader: {}", e)))?;

    let extract_dir = downloader
        .download_and_extract(&arxiv_id)
        .await
        .map_err(|e| {
            AppError::UnprocessableEntity(format!("Failed to download arXiv source: {}. The paper may not have LaTeX source available.", e))
        })?;

    info!("Downloaded and extracted arXiv source to {:?}", extract_dir);

    // Find main .tex file
    let main_tex_path = downloader
        .find_main_tex(&extract_dir)
        .map_err(|e| {
            AppError::UnprocessableEntity(format!("Failed to find main .tex file: {}", e))
        })?;

    info!("Found main .tex file: {:?}", main_tex_path);

    // Copy main.tex to paper directory as source.tex
    let source_tex_path = paper_dir.join("source.tex");
    fs::copy(&main_tex_path, &source_tex_path)
        .map_err(|e| AppError::InternalServerError(format!("Failed to copy .tex file: {}", e)))?;

    let file_path = source_tex_path.to_str().unwrap().to_string();

    // Create paper record with source URL
    let paper = Paper::create(pool, title.clone(), Some(url), file_path.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create paper in database: {:?}", e);
            AppError::InternalServerError(format!("Failed to create paper: {}", e))
        })?;

    // Read and chunk LaTeX file
    match LatexChunker::read_latex_file(&source_tex_path) {
        Ok(latex_content) => {
            let chunker = LatexChunker::default();
            let chunks = chunker.chunk(&latex_content);

            if !chunks.is_empty() {
                let chunks_len = chunks.len();
                for ch in chunks {
                    let _ = Chunk::create(
                        pool,
                        paper.id.clone(),
                        ch.index as i32,
                        ch.text,
                        ch.content_hash,
                        ch.token_count.map(|t| t as i32),
                    ).await;
                }
                info!("Prepared {} LaTeX chunks at import for paper {}", chunks_len, paper.id);
            } else {
                warn!("LaTeX chunker returned empty chunks for paper {}", paper.id);
            }
        }
        Err(e) => {
            warn!("Failed to read or chunk LaTeX file for {}: {}", paper.id, e);
        }
    }

    // Build Location header
    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        format!("/api/papers/{}", paper.id)
            .parse()
            .expect("Invalid header value"),
    );

    Ok((
        StatusCode::CREATED,
        headers,
        Json(ImportResponse {
            paper_id: paper.id,
            status: "pending".to_string(),
            message: "Paper imported successfully from LaTeX source. Chunks prepared. Next: POST /api/papers/{id}/translate, then optionally /extract-terms-jp and /scan-jp.".to_string(),
        }),
    ))
}
