use crate::api::error::AppError;
use crate::models::{Paper, PaperStatus};
use axum::{
    extract::{Query, State},
    Json,
};
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
pub struct ListResponse {
    pub papers: Vec<Paper>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
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

    Ok(Json(ListResponse {
        papers,
        total,
        page: query.page,
        limit: query.limit,
    }))
}
