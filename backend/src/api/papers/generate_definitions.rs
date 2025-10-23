use crate::api::error::AppError;
use crate::models::{Paper, PaperStatus};
use futures::FutureExt;
use std::panic::AssertUnwindSafe;
use crate::services::PaperProcessor;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct GenerateDefinitionsResponse {
    pub paper_id: String,
    pub message: String,
}

/// POST /papers/{id}/generate-definitions - Pipeline D: Generate term definitions
pub async fn generate_definitions(
    State(pool): State<SqlitePool>,
    Path(paper_id): Path<String>,
) -> Result<(StatusCode, Json<GenerateDefinitionsResponse>), AppError> {
    // Verify paper exists
    let _paper = Paper::find_by_id(&pool, &paper_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Paper {} not found", paper_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Try to acquire lock atomically; on conflict return 409
    let processor = PaperProcessor::new(pool.clone())
        .map_err(|e| AppError::InternalServerError(format!("Failed to init processor: {}", e)))?;
    let prev_status: PaperStatus = match processor.acquire_pipeline_lock(&paper_id, "generate-definitions", false).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Lock acquire failed for generate-definitions {}: {}", paper_id, e);
            return Err(AppError::Conflict(format!(
                "Another pipeline is already running for paper {}",
                paper_id
            )));
        }
    };

    // Trigger async definition generation (no-lock path)
    let pool_clone = pool.clone();
    let paper_id_clone = paper_id.clone();
    let prev_status_clone = prev_status.clone();
    tokio::spawn(async move {
        let processor = match PaperProcessor::new(pool_clone.clone()) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to create processor: {}", e);
                let _ = PaperProcessor::new(pool_clone.clone())
                    .ok()
                    .and_then(|p| p.release_pipeline_lock(&paper_id_clone, "generate-definitions", prev_status_clone.clone()).now_or_never());
                return;
            }
        };

        let result = AssertUnwindSafe(async { processor.generate_definitions_pipeline_no_lock(&paper_id_clone).await })
            .catch_unwind()
            .await;

        match result {
            Ok(Ok(())) => {
                tracing::info!("Pipeline D (generate-definitions) completed for paper {}", paper_id_clone);
            }
            Ok(Err(e)) => {
                tracing::error!("Pipeline D (generate-definitions) failed for paper {}: {}", paper_id_clone, e);
            }
            Err(_) => {
                tracing::error!("Pipeline D (generate-definitions) panicked for paper {}", paper_id_clone);
            }
        }

        if let Ok(p) = PaperProcessor::new(pool_clone.clone()) {
            let _ = p.release_pipeline_lock(&paper_id_clone, "generate-definitions", prev_status_clone).await;
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(GenerateDefinitionsResponse {
            paper_id,
            message: "Definition generation pipeline started. Use GET /papers/{id}/status to check progress.".to_string(),
        }),
    ))
}
