use crate::api::error::AppError;
use crate::models::Term;
use crate::services::llm::LlmClient;
use crate::services::terms::DefinitionGenerator;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct GenerateDefinitionResponse { pub ok: bool, pub prompt_version: String }

/// POST /terms/{id}/define - Generate or regenerate definition for a term
pub async fn generate_definition(
    State(pool): State<SqlitePool>,
    Path(term_id): Path<String>,
) -> Result<Json<GenerateDefinitionResponse>, AppError> {
    // Verify term exists
    let term = Term::find_by_id(&pool, &term_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Term {} not found", term_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Initialize definition generator
    let llm_client = LlmClient::new().map_err(|e| {
        // Map LLM initialization errors to 503 Service Unavailable
        AppError::ServiceUnavailable(format!("LLM service unavailable: {}", e))
    })?;
    let def_generator = DefinitionGenerator::new(llm_client, pool);

    // Generate and store definition (v2)
    def_generator
        .generate_and_store_v2(None, &term.id, &term.lemma_en, &term.lemma_ja, None)
        .await
        .map_err(|e| {
            // Map LLM generation errors to 503 Service Unavailable
            if e.to_string().contains("LLM") || e.to_string().contains("timeout") {
                AppError::ServiceUnavailable(format!("LLM service error: {}", e))
            } else {
                AppError::InternalServerError(format!("Definition generation failed: {}", e))
            }
        })?;

    Ok(Json(GenerateDefinitionResponse { ok: true, prompt_version: "d2".into() }))
}
