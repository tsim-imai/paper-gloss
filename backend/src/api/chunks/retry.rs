use crate::api::error::AppError;
use crate::models::Chunk;
use crate::services::PaperProcessor;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

/// POST /chunks/{id}/retry - Retry translation for a failed chunk
#[derive(Debug, Serialize)]
pub struct RetryAcceptedResponse {
    pub chunk_id: String,
    pub status: String,
    pub message: String,
}

pub async fn retry_chunk(
    State(pool): State<SqlitePool>,
    Path(chunk_id): Path<String>,
) -> Result<(StatusCode, Json<RetryAcceptedResponse>), AppError> {
    // Verify chunk exists
    let chunk = Chunk::find_by_id(&pool, &chunk_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Chunk {} not found", chunk_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Trigger async retry
    let pool_clone = pool.clone();
    let chunk_id_clone = chunk_id.clone();
    tokio::spawn(async move {
        let processor = match PaperProcessor::new(pool_clone) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to create processor: {}", e);
                return;
            }
        };

        if let Err(e) = processor.retry_chunk(&chunk_id_clone).await {
            tracing::error!("Chunk retry failed: {}", e);
        }
    });

    // Return 202 Accepted to indicate async retry started
    Ok((
        StatusCode::ACCEPTED,
        Json(RetryAcceptedResponse {
            chunk_id: chunk.id,
            status: "accepted".to_string(),
            message: "Chunk retry has been scheduled".to_string(),
        }),
    ))
}
