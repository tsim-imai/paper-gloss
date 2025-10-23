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
pub struct TranslateTriggerResponse {
    pub paper_id: String,
    pub message: String,
}

/// POST /papers/{id}/translate - Pipeline A: Translate all chunks
pub async fn translate_paper(
    State(pool): State<SqlitePool>,
    Path(paper_id): Path<String>,
) -> Result<(StatusCode, Json<TranslateTriggerResponse>), AppError> {
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

    let prev_status: PaperStatus = match processor.acquire_pipeline_lock(&paper_id, "translate", true).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Lock acquire failed for translate {}: {}", paper_id, e);
            return Err(AppError::Conflict(format!(
                "Another pipeline is already running for paper {}",
                paper_id
            )));
        }
    };

    // Trigger async translation (no-lock path)
    let pool_clone = pool.clone();
    let paper_id_clone = paper_id.clone();
    let prev_status_clone = prev_status.clone();
    tokio::spawn(async move {
        let processor = match PaperProcessor::new(pool_clone.clone()) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to create processor: {}", e);
                // release lock on failure to construct
                let _ = PaperProcessor::new(pool_clone.clone())
                    .ok()
                    .and_then(|p| p.release_pipeline_lock(&paper_id_clone, "translate", prev_status_clone.clone()).now_or_never());
                return;
            }
        };

        let result = AssertUnwindSafe(async { processor.translate_paper_no_lock(&paper_id_clone).await })
            .catch_unwind()
            .await;

        match result {
            Ok(Ok(())) => {
                tracing::info!("Pipeline A (translation) completed for paper {}", paper_id_clone);
            }
            Ok(Err(e)) => {
                tracing::error!("Pipeline A (translation) failed for paper {}: {}", paper_id_clone, e);
            }
            Err(_) => {
                tracing::error!("Pipeline A (translation) panicked for paper {}", paper_id_clone);
            }
        }

        // Always release lock
        if let Ok(p) = PaperProcessor::new(pool_clone.clone()) {
            let _ = p.release_pipeline_lock(&paper_id_clone, "translate", prev_status_clone).await;
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(TranslateTriggerResponse {
            paper_id,
            message: "Translation pipeline started. Use GET /papers/{id}/status to check progress.".to_string(),
        }),
    ))
}
