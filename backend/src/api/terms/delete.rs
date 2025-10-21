use crate::api::error::AppError;
use crate::models::Term;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::SqlitePool;

/// DELETE /terms/{id} - Delete a term
pub async fn delete_term(
    State(pool): State<SqlitePool>,
    Path(term_id): Path<String>,
) -> Result<StatusCode, AppError> {
    // Verify term exists
    let _term = Term::find_by_id(&pool, &term_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Term {} not found", term_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Delete term (cascades to variants, definitions, occurrences)
    Term::delete(&pool, &term_id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete term: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}
