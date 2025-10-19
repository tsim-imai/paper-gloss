use crate::api::error::AppError;
use crate::models::{Chunk, Paper};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct ProcessingProgress {
    pub extraction: String,
    pub translation: TranslationProgress,
    pub term_extraction: String,
    pub definitions: DefinitionProgress,
}

#[derive(Debug, Serialize)]
pub struct TranslationProgress {
    pub total_chunks: i64,
    pub completed_chunks: i64,
    pub failed_chunks: i64,
}

#[derive(Debug, Serialize)]
pub struct DefinitionProgress {
    pub total_terms: i64,
    pub completed_definitions: i64,
}

#[derive(Debug, Serialize)]
pub struct ProcessingStatusResponse {
    pub paper_id: String,
    pub status: String,
    pub progress: ProcessingProgress,
}

/// GET /papers/{id}/status - Get processing status and progress for a paper
pub async fn get_paper_status(
    State(pool): State<SqlitePool>,
    Path(paper_id): Path<String>,
) -> Result<Json<ProcessingStatusResponse>, AppError> {
    // Verify paper exists
    let paper = Paper::find_by_id(&pool, &paper_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Paper {} not found", paper_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Get current progress
    let total_chunks = Chunk::count_by_paper_id(&pool, &paper_id).await.unwrap_or(0);
    let completed_chunks = Chunk::count_translated_by_paper_id(&pool, &paper_id)
        .await
        .unwrap_or(0);
    let failed_chunks = Chunk::count_failed_by_paper_id(&pool, &paper_id)
        .await
        .unwrap_or(0);

    Ok(Json(ProcessingStatusResponse {
        paper_id,
        status: paper.status.to_string(),
        progress: ProcessingProgress {
            extraction: if total_chunks > 0 {
                "completed".to_string()
            } else {
                "pending".to_string()
            },
            translation: TranslationProgress {
                total_chunks,
                completed_chunks,
                failed_chunks,
            },
            term_extraction: "pending".to_string(), // Will be implemented in Phase 4
            definitions: DefinitionProgress {
                total_terms: 0,         // Will be implemented in Phase 4
                completed_definitions: 0, // Will be implemented in Phase 4
            },
        },
    }))
}
