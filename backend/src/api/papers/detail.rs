use crate::api::error::AppError;
use crate::models::Paper;
use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::SqlitePool;

/// GET /papers/{id} - Get paper details and processing status
pub async fn get_paper(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Paper>, AppError> {
    let paper = Paper::find_by_id(&pool, &id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Paper {} not found", id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    Ok(Json(paper))
}
