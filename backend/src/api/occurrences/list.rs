use crate::api::error::AppError;
use crate::models::Occurrence;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct OccurrencesQuery {
    pub paper_id: Option<String>,
    pub term_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OccurrenceResponse {
    pub id: String,
    pub term_id: String,
    pub paper_id: String,
    pub chunk_id: String,
    pub start_pos: i32,
    pub end_pos: i32,
    pub surface: Option<String>,
    pub method: String,
    pub variant_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct OccurrencesListResponse {
    pub occurrences: Vec<OccurrenceResponse>,
    pub total: usize,
}

/// GET /occurrences - List occurrences filtered by paper_id or term_id
pub async fn list_occurrences(
    State(pool): State<SqlitePool>,
    Query(query): Query<OccurrencesQuery>,
) -> Result<Json<OccurrencesListResponse>, AppError> {
    let occurrences = if let Some(paper_id) = query.paper_id {
        // Filter by paper
        Occurrence::find_by_paper_id(&pool, &paper_id)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
    } else if let Some(term_id) = query.term_id {
        // Filter by term
        Occurrence::find_by_term_id(&pool, &term_id)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
    } else {
        // No filter - return error (avoid full table scan)
        return Err(AppError::BadRequest(
            "Must provide either paper_id or term_id query parameter".to_string(),
        ));
    };

    let total = occurrences.len();

    let occurrence_responses: Vec<OccurrenceResponse> = occurrences
        .into_iter()
        .map(|o| OccurrenceResponse {
            id: o.id,
            term_id: o.term_id,
            paper_id: o.paper_id,
            chunk_id: o.chunk_id,
            start_pos: o.start_pos,
            end_pos: o.end_pos,
            surface: o.surface,
            method: o.method,
            variant_id: o.variant_id,
            created_at: o.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(OccurrencesListResponse {
        occurrences: occurrence_responses,
        total,
    }))
}
