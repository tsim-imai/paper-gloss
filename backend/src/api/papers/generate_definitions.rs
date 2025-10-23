use crate::api::error::AppError;
use crate::models::Paper;
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

    // If another pipeline is running, return 409 Conflict
    let lock_exists: Option<i64> = sqlx::query_scalar(
        r#"SELECT 1 FROM pipeline_locks WHERE paper_id = ? LIMIT 1"#,
    )
    .bind(&paper_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    if lock_exists.is_some() {
        return Err(AppError::Conflict(format!(
            "Another pipeline is already running for paper {}",
            paper_id
        )));
    }

    // Trigger async definition generation
    let pool_clone = pool.clone();
    let paper_id_clone = paper_id.clone();
    tokio::spawn(async move {
        let processor = match PaperProcessor::new(pool_clone.clone()) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to create processor: {}", e);
                return;
            }
        };

        let result = AssertUnwindSafe(async { processor.generate_definitions_pipeline(&paper_id_clone).await })
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
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(GenerateDefinitionsResponse {
            paper_id,
            message: "Definition generation pipeline started. Use GET /papers/{id}/status to check progress.".to_string(),
        }),
    ))
}
