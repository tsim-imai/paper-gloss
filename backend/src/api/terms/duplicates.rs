use crate::api::error::AppError;
use crate::services::terms::DuplicateDetectionService;
use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct TermInfo {
    pub id: String,
    pub lemma_en: String,
    pub lemma_ja: String,
    pub reading_kana: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DuplicatePairResponse {
    pub term1: TermInfo,
    pub term2: TermInfo,
    pub similarity_score: f64,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct DuplicatesResponse {
    pub duplicates: Vec<DuplicatePairResponse>,
    pub total: usize,
}

/// GET /terms/duplicates - Find potential duplicate terms
pub async fn find_duplicates(
    State(pool): State<SqlitePool>,
) -> Result<Json<DuplicatesResponse>, AppError> {
    let service = DuplicateDetectionService::new(pool);

    let pairs = service
        .find_duplicates()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Duplicate detection failed: {}", e)))?;

    let responses: Vec<DuplicatePairResponse> = pairs
        .into_iter()
        .map(|pair| DuplicatePairResponse {
            term1: TermInfo {
                id: pair.term1.id,
                lemma_en: pair.term1.lemma_en,
                lemma_ja: pair.term1.lemma_ja,
                reading_kana: pair.term1.reading_kana,
            },
            term2: TermInfo {
                id: pair.term2.id,
                lemma_en: pair.term2.lemma_en,
                lemma_ja: pair.term2.lemma_ja,
                reading_kana: pair.term2.reading_kana,
            },
            similarity_score: pair.similarity_score,
            reason: pair.reason,
        })
        .collect();

    let total = responses.len();

    Ok(Json(DuplicatesResponse {
        duplicates: responses,
        total,
    }))
}
