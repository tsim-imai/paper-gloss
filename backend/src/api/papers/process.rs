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
pub struct ProcessTriggerResponse {
    pub paper_id: String,
    pub message: String,
}

/// POST /papers/{id}/process - Trigger async processing for a paper
pub async fn process_paper(
    State(pool): State<SqlitePool>,
    Path(paper_id): Path<String>,
) -> Result<(StatusCode, Json<ProcessTriggerResponse>), AppError> {
    // Verify paper exists
    let _paper = Paper::find_by_id(&pool, &paper_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Paper {} not found", paper_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Trigger async processing
    let pool_clone = pool.clone();
    let paper_id_clone = paper_id.clone();
    tokio::spawn(async move {
        let processor = match PaperProcessor::new(pool_clone.clone()) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to create processor: {}", e);
                // mark failed
                let _ = Paper::update_status(&pool_clone, &paper_id_clone, PaperStatus::Failed).await;
                return;
            }
        };

        let result = AssertUnwindSafe(async { processor.process_paper(&paper_id_clone).await })
            .catch_unwind()
            .await;

        match result {
            Ok(Ok(())) => { /* done */ }
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

    Ok((
        StatusCode::ACCEPTED,
        Json(ProcessTriggerResponse {
            paper_id,
            message: "Processing started. Use GET /papers/{id}/status to check progress.".to_string(),
        }),
    ))
}
