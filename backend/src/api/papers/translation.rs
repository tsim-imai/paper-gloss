use crate::api::error::AppError;
use crate::models::Chunk;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct ChunkResponse {
    pub id: String,
    pub paper_id: String,
    pub chunk_index: i32,
    pub original_text: String,
    pub translated_text: Option<String>,
    pub content_hash: String,
    pub status: String,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct TranslationResponse {
    pub paper_id: String,
    pub chunks: Vec<ChunkResponse>,
}

/// GET /papers/{id}/translation - Get all chunks with translations for a paper
pub async fn get_translation(
    State(pool): State<SqlitePool>,
    Path(paper_id): Path<String>,
) -> Result<Json<TranslationResponse>, AppError> {
    // Get all chunks for the paper
    let chunks = Chunk::find_by_paper_id(&pool, &paper_id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    if chunks.is_empty() {
        return Err(AppError::NotFound(format!(
            "No chunks found for paper {}",
            paper_id
        )));
    }

    // Transform all chunks to response format
    let chunk_responses: Vec<ChunkResponse> = chunks
        .into_iter()
        .map(|chunk| {
            let has_translation = chunk.trans_html.is_some();
            let status = if has_translation {
                "translated"
            } else {
                "pending"
            };

            ChunkResponse {
                id: chunk.id,
                paper_id: chunk.paper_id,
                chunk_index: chunk.index,
                original_text: chunk.src_text,
                translated_text: chunk.trans_html,
                content_hash: chunk.content_hash,
                status: status.to_string(),
                retry_count: 0, // Not implemented yet
                error_message: None, // Not implemented yet
                created_at: chunk.created_at.to_rfc3339(),
                updated_at: chunk.updated_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(TranslationResponse {
        paper_id,
        chunks: chunk_responses,
    }))
}
