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
use serde::Deserialize;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct ExtractTermsJpResponse {
    pub paper_id: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtractTermsJpRequest {
    pub max_terms: Option<i64>,
    pub min_confidence: Option<f64>,
}

/// POST /papers/{id}/extract-terms-jp - Pipeline B: Extract Japanese terms (v2: POSなし、variants/aliases保存)
pub async fn extract_terms_jp(
    State(pool): State<SqlitePool>,
    Path(paper_id): Path<String>,
    req: Option<Json<ExtractTermsJpRequest>>,
) -> Result<(StatusCode, Json<ExtractTermsJpResponse>), AppError> {
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
    let prev_status: PaperStatus = match processor.acquire_pipeline_lock(&paper_id, "extract-terms-jp", false).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Lock acquire failed for extract-terms-jp {}: {}", paper_id, e);
            return Err(AppError::Conflict(format!(
                "Another pipeline is already running for paper {}",
                paper_id
            )));
        }
    };

    // Trigger async term extraction (no-lock path)
    let pool_clone = pool.clone();
    let paper_id_clone = paper_id.clone();
    let prev_status_clone = prev_status.clone();
    let (max_terms, min_confidence) = match req {
        Some(Json(r)) => (r.max_terms, r.min_confidence),
        None => (None, None),
    };
    tokio::spawn(async move {
        let processor = match PaperProcessor::new(pool_clone.clone()) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to create processor: {}", e);
                let _ = PaperProcessor::new(pool_clone.clone())
                    .ok()
                    .and_then(|p| p.release_pipeline_lock(&paper_id_clone, "extract-terms-jp", prev_status_clone.clone()).now_or_never());
                return;
            }
        };

        let result = AssertUnwindSafe(async { processor.extract_terms_jp_no_lock(&paper_id_clone, min_confidence, max_terms).await })
            .catch_unwind()
            .await;

        match result {
            Ok(Ok(())) => {
                tracing::info!("Pipeline B (extract-terms-jp) completed for paper {}", paper_id_clone);
            }
            Ok(Err(e)) => {
                tracing::error!("Pipeline B (extract-terms-jp) failed for paper {}: {}", paper_id_clone, e);
            }
            Err(_) => {
                tracing::error!("Pipeline B (extract-terms-jp) panicked for paper {}", paper_id_clone);
            }
        }

        if let Ok(p) = PaperProcessor::new(pool_clone.clone()) {
            let _ = p.release_pipeline_lock(&paper_id_clone, "extract-terms-jp", prev_status_clone).await;
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(ExtractTermsJpResponse {
            paper_id,
            message: "Term extraction pipeline started. Use GET /papers/{id}/status to check progress.".to_string(),
        }),
    ))
}
