use crate::api::error::AppError;
use crate::models::{Term, TermVariant, Definition};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct TermDetailResponse {
    pub id: String,
    pub slug: String,
    pub lemma_en: String,
    pub lemma_ja: String,
    pub reading_kana: Option<String>,
    pub pos: Option<String>,
    pub tags: Option<String>,
    pub note: Option<String>,
    pub definition: Option<DefinitionResponse>,
    pub variants: Vec<VariantResponse>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct DefinitionResponse {
    pub text: String,
    pub provider: String,
}

#[derive(Debug, Serialize)]
pub struct VariantResponse {
    pub lang: String,
    pub surface: String,
}

/// GET /terms/{id} - Get term details with definition and variants
pub async fn get_term_detail(
    State(pool): State<SqlitePool>,
    Path(term_id): Path<String>,
) -> Result<Json<TermDetailResponse>, AppError> {
    // Get term
    let term = Term::find_by_id(&pool, &term_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Term {} not found", term_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Get definition
    let definition = Definition::find_by_term_id(&pool, &term_id)
        .await
        .ok()
        .flatten()
        .map(|d| DefinitionResponse {
            text: d.text,
            provider: d.provider,
        });

    // Get variants
    let variant_models = TermVariant::find_by_term_id(&pool, &term_id)
        .await
        .unwrap_or_default();

    let variants = variant_models
        .into_iter()
        .map(|v| VariantResponse {
            lang: v.lang,
            surface: v.surface,
        })
        .collect();

    Ok(Json(TermDetailResponse {
        id: term.id,
        slug: term.slug,
        lemma_en: term.lemma_en,
        lemma_ja: term.lemma_ja,
        reading_kana: term.reading_kana,
        pos: term.pos,
        tags: term.tags,
        note: term.note,
        definition,
        variants,
        created_at: term.created_at.to_rfc3339(),
        updated_at: term.updated_at.to_rfc3339(),
    }))
}
