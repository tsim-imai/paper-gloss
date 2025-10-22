use crate::api::error::AppError;
use crate::models::{Paper, PaperStatus};
use crate::services::PaperProcessor;
use axum::{
    extract::{Multipart, State},
    http::{header, HeaderMap, StatusCode},
    Json,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};
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
    use crate::services::pdf::PdfExtractor;
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

    // Trigger async processing
    let pool_clone = pool.clone();
    let paper_id_clone = paper.id.clone();
    tokio::spawn(async move {
        let processor = match PaperProcessor::new(pool_clone.clone()) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to create processor: {}", e);
                let _ = Paper::update_status(&pool_clone, &paper_id_clone, PaperStatus::Failed).await;
                return;
            }
        };

        let result = AssertUnwindSafe(async { processor.process_paper(&paper_id_clone).await })
            .catch_unwind()
            .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                tracing::error!("Paper processing failed: {}", e);
                let _ = Paper::update_status(&pool_clone, &paper_id_clone, PaperStatus::Failed).await;
            }
            Err(_) => {
                tracing::error!("Paper processing panicked");
                let _ = Paper::update_status(&pool_clone, &paper_id_clone, PaperStatus::Failed).await;
            }
        }
    });

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
            status: "processing".to_string(),
            message: "Paper imported successfully. Processing started.".to_string(),
        }),
    ))
}

/// Handle arXiv URL import (data-model.md:326-454)
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

    // Convert /abs/ to /pdf/
    let pdf_url = url.replace("/abs/", "/pdf/") + ".pdf";

    debug!("Downloading PDF from {}", pdf_url);

    // Download PDF with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::InternalServerError(format!("HTTP client error: {}", e)))?;

    let response = client
        .get(&pdf_url)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::GatewayTimeout("arXiv download timed out. Please retry later.".to_string())
            } else {
                AppError::UnprocessableEntity(format!("Failed to download PDF: {}", e))
            }
        })?;

    if !response.status().is_success() {
        return Err(AppError::UnprocessableEntity(
            "PDF not found at arXiv. Verify the paper ID is correct.".to_string(),
        ));
    }

    // Check Content-Type
    if let Some(content_type) = response.headers().get("content-type") {
        if !content_type.to_str().unwrap_or("").contains("application/pdf") {
            return Err(AppError::UnprocessableEntity(
                "Downloaded file is not a valid PDF".to_string(),
            ));
        }
    }

    let pdf_bytes = response.bytes().await.map_err(|e| {
        AppError::UnprocessableEntity(format!("Failed to read PDF data: {}", e))
    })?;

    // Check size
    if pdf_bytes.len() > 100 * 1024 * 1024 {
        return Err(AppError::UnprocessableEntity(
            "PDF file too large (max 100 MB)".to_string(),
        ));
    }

    // Generate paper ID and save
    let paper_id = Uuid::new_v4().to_string();
    let file_path = format!("artifacts/papers/{}/source.pdf", paper_id);

    fs::create_dir_all(format!("artifacts/papers/{}", paper_id))
        .map_err(|e| AppError::InternalServerError(format!("Failed to create directory: {}", e)))?;

    fs::write(&file_path, &pdf_bytes)
        .map_err(|e| AppError::InternalServerError(format!("Failed to write PDF: {}", e)))?;

    info!("Downloaded and saved PDF from arXiv to {}", file_path);

    // Create paper record with source URL
    let paper = Paper::create(pool, title.clone(), Some(url), file_path.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create paper in database: {:?}", e);
            AppError::InternalServerError(format!("Failed to create paper: {}", e))
        })?;

    // Trigger async processing
    let pool_clone = pool.clone();
    let paper_id_clone = paper.id.clone();
    tokio::spawn(async move {
        let processor = match PaperProcessor::new(pool_clone.clone()) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to create processor: {}", e);
                let _ = Paper::update_status(&pool_clone, &paper_id_clone, PaperStatus::Failed).await;
                return;
            }
        };

        let result = AssertUnwindSafe(async { processor.process_paper(&paper_id_clone).await })
            .catch_unwind()
            .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                tracing::error!("Paper processing failed: {}", e);
                let _ = Paper::update_status(&pool_clone, &paper_id_clone, PaperStatus::Failed).await;
            }
            Err(_) => {
                tracing::error!("Paper processing panicked");
                let _ = Paper::update_status(&pool_clone, &paper_id_clone, PaperStatus::Failed).await;
            }
        }
    });

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
            status: "processing".to_string(),
            message: "Paper imported successfully. Processing started.".to_string(),
        }),
    ))
}
use futures::FutureExt;
use std::panic::AssertUnwindSafe;
