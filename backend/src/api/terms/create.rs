use crate::api::error::AppError;
use crate::models::{Term, TermVariant};
use crate::services::terms::{generate_english_variants, generate_japanese_variants};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct CreateTermRequest {
    pub lemma_en: String,
    pub lemma_ja: String,
    pub reading_kana: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateTermResponse {
    pub id: String,
    pub slug: String,
    pub lemma_en: String,
    pub lemma_ja: String,
    pub reading_kana: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// POST /terms - Create a new term
pub async fn create_term(
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateTermRequest>,
) -> Result<(StatusCode, Json<CreateTermResponse>), AppError> {
    // Generate slug from lemma_en (lowercase, replace spaces with hyphens)
    let slug = req
        .lemma_en
        .to_lowercase()
        .replace(' ', "-")
        .replace('_', "-");

    // Create term
    let term = Term::create(
        &pool,
        slug,
        req.lemma_en.clone(),
        req.lemma_ja.clone(),
        req.reading_kana.clone(),
        req.note.clone(),
    )
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to create term: {}", e)))?;

    // Generate and store variants
    let en_variants = generate_english_variants(&req.lemma_en);
    for variant in en_variants {
        let _ = TermVariant::create(&pool, term.id.clone(), "en".to_string(), variant).await;
    }

    let ja_variants = generate_japanese_variants(&req.lemma_ja);
    for variant in ja_variants {
        let _ = TermVariant::create(&pool, term.id.clone(), "ja".to_string(), variant).await;
    }

    Ok((
        StatusCode::CREATED,
        Json(CreateTermResponse {
            id: term.id,
            slug: term.slug,
            lemma_en: term.lemma_en,
            lemma_ja: term.lemma_ja,
            reading_kana: term.reading_kana,
            note: term.note,
            created_at: term.created_at.to_rfc3339(),
            updated_at: term.updated_at.to_rfc3339(),
        }),
    ))
}
