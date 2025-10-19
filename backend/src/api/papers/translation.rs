use crate::api::error::AppError;
use crate::models::Chunk;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct TranslatedChunk {
    pub chunk_id: String,
    pub index: i32,
    pub trans_html: String,
}

#[derive(Debug, Serialize)]
pub struct TranslationResponse {
    pub paper_id: String,
    pub chunks: Vec<TranslatedChunk>,
}

/// GET /papers/{id}/translation - Get translated text for a paper
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

    // Filter only translated chunks and transform
    let translated_chunks: Vec<TranslatedChunk> = chunks
        .into_iter()
        .filter_map(|chunk| {
            chunk.trans_html.map(|trans_html| TranslatedChunk {
                chunk_id: chunk.id,
                index: chunk.index,
                trans_html,
            })
        })
        .collect();

    Ok(Json(TranslationResponse {
        paper_id,
        chunks: translated_chunks,
    }))
}
