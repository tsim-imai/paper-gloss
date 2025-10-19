use crate::api::error::AppError;
use crate::services::terms::TermMergeService;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct MergeTermsRequest {
    pub source_id: String,
    pub target_id: String,
    pub confirmed: bool,
}

#[derive(Debug, Serialize)]
pub struct MergeTermsResponse {
    pub source_id: String,
    pub target_id: String,
    pub affected_count: i64,
    pub message: String,
}

/// POST /terms/merge - Merge two terms
pub async fn merge_terms(
    State(pool): State<SqlitePool>,
    Json(req): Json<MergeTermsRequest>,
) -> Result<Json<MergeTermsResponse>, AppError> {
    if !req.confirmed {
        return Err(AppError::BadRequest(
            "Merge operation requires confirmation".to_string(),
        ));
    }

    let merge_service = TermMergeService::new(pool);

    let affected_count = merge_service
        .merge_terms(&req.source_id, &req.target_id, req.confirmed)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Merge failed: {}", e)))?;

    Ok(Json(MergeTermsResponse {
        source_id: req.source_id,
        target_id: req.target_id,
        affected_count,
        message: format!(
            "Successfully merged terms, {} entities updated",
            affected_count
        ),
    }))
}
