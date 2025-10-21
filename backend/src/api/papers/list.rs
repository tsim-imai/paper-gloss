use crate::api::error::AppError;
use crate::models::{Chunk, Paper, PaperStatus};
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct PaperListItem {
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

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub items: Vec<PaperListItem>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

/// Extract arXiv ID from URL
fn extract_arxiv_id(url: &str) -> Option<String> {
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

/// GET /papers - List all papers with optional status filter
pub async fn list_papers(
    State(pool): State<SqlitePool>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    // Parse status if provided
    let status = if let Some(status_str) = query.status {
        Some(match status_str.as_str() {
            "pending" => PaperStatus::Pending,
            "processing" => PaperStatus::Processing,
            "completed" => PaperStatus::Completed,
            "failed" => PaperStatus::Failed,
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Invalid status: {}. Must be one of: pending, processing, completed, failed",
                    status_str
                )))
            }
        })
    } else {
        None
    };

    // Get papers
    let papers = Paper::list(&pool, status.clone(), query.page, query.limit)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Get total count
    let total = Paper::count(&pool, status)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Calculate total pages
    let total_pages = (total as f64 / query.limit as f64).ceil() as i64;

    // Convert to list items with source and arxiv_id
    let mut items = Vec::new();
    for paper in papers {
        let (source, arxiv_id) = if let Some(ref url) = paper.source_url {
            ("url".to_string(), extract_arxiv_id(url))
        } else {
            ("upload".to_string(), None)
        };

        // Get chunk statistics
        let chunks = Chunk::find_by_paper_id(&pool, &paper.id).await.ok();
        let (total_chunks, translated_chunks, failed_chunks) = if let Some(ref chunks) = chunks {
            let total = chunks.len() as i64;
            let translated = chunks.iter().filter(|c| c.trans_html.is_some()).count() as i64;
            let failed = chunks.iter().filter(|c| c.status == "failed").count() as i64;
            (Some(total), Some(translated), Some(failed))
        } else {
            (None, None, None)
        };

        items.push(PaperListItem {
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
        });
    }

    Ok(Json(ListResponse {
        items,
        total,
        page: query.page,
        limit: query.limit,
        total_pages,
    }))
}
