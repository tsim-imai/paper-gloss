use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use sqlx::SqlitePool;
use tokio::fs;

use crate::api::error::AppError;
use crate::models::paper::Paper;

/// GET /api/papers/:id/file - Serve PDF file
pub async fn get_paper_file(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Get paper from database
    let paper = Paper::find_by_id(&pool, &id)
        .await
        .map_err(|_| AppError::NotFound("Paper not found".to_string()))?;

    // Read PDF file
    let file_bytes = fs::read(&paper.file_path)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read PDF file: {}", e);
            AppError::NotFound("PDF file not found".to_string())
        })?;

    // Return PDF with proper content type
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/pdf")],
        file_bytes,
    ))
}
