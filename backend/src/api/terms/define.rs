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
pub struct GenerateDefinitionResponse {
    pub term_id: String,
    pub definition: String,
    pub message: String,
}

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
    let llm_client = LlmClient::new()
        .map_err(|e| AppError::InternalServerError(format!("LLM client error: {}", e)))?;
    let def_generator = DefinitionGenerator::new(llm_client, pool);

    // Generate and store definition
    let definition = def_generator
        .generate_and_store(&term.id, &term.lemma_en, &term.lemma_ja, None)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Definition generation failed: {}", e)))?;

    Ok(Json(GenerateDefinitionResponse {
        term_id: term.id,
        definition,
        message: "Definition generated successfully".to_string(),
    }))
}
