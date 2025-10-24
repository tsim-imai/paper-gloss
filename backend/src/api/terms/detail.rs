use crate::api::error::AppError;
use crate::models::{Term, TermVariant, Definition, TermAlias, DefinitionMeta};
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
    pub note: Option<String>,
    pub definition: Option<DefinitionResponse>,
    pub definition_meta: Option<DefinitionMetaResponse>,
    pub variants: Vec<VariantResponse>,
    pub aliases: Vec<AliasResponse>,
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

#[derive(Debug, Serialize)]
pub struct AliasResponse {
    pub lang: String,
    pub surface: String,
    pub kind: String,
    pub confidence: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct DefinitionMetaResponse {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub prompt_version: Option<String>,
    pub confidence: Option<f64>,
    pub flags: Option<Vec<String>>,
    pub updated_at: String,
}

/// GET /terms/{id} - Get term details with definition and variants (POS/tags removed in v2)
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

    // Get definition meta
    let definition_meta = DefinitionMeta::find_by_term_id(&pool, &term_id)
        .await
        .ok()
        .flatten()
        .map(|m| {
            let flags: Option<Vec<String>> = m
                .flags
                .as_ref()
                .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok());
            DefinitionMetaResponse {
                provider: m.provider,
                model: m.model,
                prompt_version: m.prompt_version,
                confidence: m.confidence,
                flags,
                updated_at: m.updated_at.to_rfc3339(),
            }
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

    // Get aliases
    let alias_models = TermAlias::find_by_term_id(&pool, &term_id)
        .await
        .unwrap_or_default();
    let aliases = alias_models
        .into_iter()
        .map(|a| AliasResponse { lang: a.lang, surface: a.surface, kind: a.kind, confidence: a.confidence })
        .collect();

    Ok(Json(TermDetailResponse {
        id: term.id,
        slug: term.slug,
        lemma_en: term.lemma_en,
        lemma_ja: term.lemma_ja,
        reading_kana: term.reading_kana,
        note: term.note,
        definition,
        definition_meta,
        variants,
        aliases,
        created_at: term.created_at.to_rfc3339(),
        updated_at: term.updated_at.to_rfc3339(),
    }))
}
