use crate::api::error::AppError;
use crate::models::Term;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct UpdateTermRequest {
    pub lemma_en: Option<String>,
    pub lemma_ja: Option<String>,
    pub reading_kana: Option<String>,
    pub pos: Option<String>,
    pub tags: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateTermResponse {
    pub id: String,
    pub message: String,
}

/// PATCH /terms/{id} - Update a term
pub async fn update_term(
    State(pool): State<SqlitePool>,
    Path(term_id): Path<String>,
    Json(req): Json<UpdateTermRequest>,
) -> Result<Json<UpdateTermResponse>, AppError> {
    // Verify term exists
    let _term = Term::find_by_id(&pool, &term_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Term {} not found", term_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Update term
    Term::update(
        &pool,
        &term_id,
        req.lemma_en,
        req.lemma_ja,
        req.reading_kana,
        req.pos,
        req.tags,
        req.note,
    )
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to update term: {}", e)))?;

    Ok(Json(UpdateTermResponse {
        id: term_id,
        message: "Term updated successfully".to_string(),
    }))
}
