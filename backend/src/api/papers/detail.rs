use crate::api::error::AppError;
use crate::models::{Chunk, Paper, PaperStatus};
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct PaperDetailResponse {
    pub id: String,
    pub title: String,
    pub source: String,  // 'upload' or 'url'
    pub arxiv_id: Option<String>,
    pub file_path: String,
    pub status: PaperStatus,
    pub total_chunks: Option<i64>,
    pub translated_chunks: Option<i64>,
    pub failed_chunks: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Extract arXiv ID from URL
fn extract_arxiv_id(url: &str) -> Option<String> {
    // Match patterns like:
    // - https://arxiv.org/abs/2301.12345
    // - https://arxiv.org/pdf/2301.12345.pdf
    if let Some(abs_idx) = url.find("/abs/") {
        let id_start = abs_idx + 5;
        Some(url[id_start..].split('/').next()?.to_string())
    } else if let Some(pdf_idx) = url.find("/pdf/") {
        let id_start = pdf_idx + 5;
        Some(url[id_start..].trim_end_matches(".pdf").to_string())
    } else {
        None
    }
}

/// GET /papers/{id} - Get paper details and processing status
pub async fn get_paper(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<PaperDetailResponse>, AppError> {
    let paper = Paper::find_by_id(&pool, &id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Paper {} not found", id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Determine source and arxiv_id
    let (source, arxiv_id) = if let Some(ref url) = paper.source_url {
        ("url".to_string(), extract_arxiv_id(url))
    } else {
        ("upload".to_string(), None)
    };

    // Get chunk statistics
    let chunks = Chunk::find_by_paper_id(&pool, &id).await.ok();
    let (total_chunks, translated_chunks, failed_chunks) = if let Some(ref chunks) = chunks {
        let total = chunks.len() as i64;
        let translated = chunks.iter().filter(|c| c.trans_html.is_some()).count() as i64;
        let failed = chunks.iter().filter(|c| c.status == "failed").count() as i64;
        (Some(total), Some(translated), Some(failed))
    } else {
        (None, None, None)
    };

    Ok(Json(PaperDetailResponse {
        id: paper.id,
        title: paper.title,
        source,
        arxiv_id,
        file_path: paper.file_path,
        status: paper.status,
        total_chunks,
        translated_chunks,
        failed_chunks,
        created_at: paper.created_at,
        updated_at: paper.updated_at,
    }))
}
