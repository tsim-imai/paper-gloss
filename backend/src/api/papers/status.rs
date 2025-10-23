use crate::api::error::AppError;
use crate::models::{Chunk, Paper, Occurrence};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct TranslationProgress {
    pub total_chunks: i64,
    pub completed_chunks: i64,
    pub failed_chunks: i64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TermsJpProgress {
    pub total_terms: i64,
    pub last_run_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ScanJpProgress {
    pub total_occurrences: i64,
    pub last_run_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct DefinitionsProgress {
    pub generated: i64,
    pub failed: i64,
    pub last_run_at: Option<String>,
    pub status: String,
    pub result_state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProcessingStatusResponse {
    pub paper_id: String,
    pub status: String,
    pub translation: TranslationProgress,
    pub terms_jp: TermsJpProgress,
    pub scan_jp: ScanJpProgress,
    pub definitions: DefinitionsProgress,
}

/// GET /papers/{id}/status - Get processing status and progress for a paper
pub async fn get_paper_status(
    State(pool): State<SqlitePool>,
    Path(paper_id): Path<String>,
) -> Result<Json<ProcessingStatusResponse>, AppError> {
    // Verify paper exists
    let paper = Paper::find_by_id(&pool, &paper_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Paper {} not found", paper_id)),
            _ => AppError::InternalServerError(format!("Database error: {}", e)),
        })?;

    // Check which pipelines are currently running by checking pipeline_locks
    let active_pipeline: Option<String> = sqlx::query_scalar(
        r#"SELECT pipeline FROM pipeline_locks WHERE paper_id = ? LIMIT 1"#,
    )
    .bind(&paper_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    // Translation progress
    let total_chunks = Chunk::count_by_paper_id(&pool, &paper_id).await.unwrap_or(0);
    let completed_chunks = Chunk::count_translated_by_paper_id(&pool, &paper_id).await.unwrap_or(0);
    let failed_chunks = Chunk::count_failed_by_paper_id(&pool, &paper_id).await.unwrap_or(0);

    let translation_status = if active_pipeline.as_deref() == Some("translate") {
        "processing"
    } else if completed_chunks == total_chunks && total_chunks > 0 {
        "completed"
    } else if failed_chunks == total_chunks && total_chunks > 0 {
        "failed"
    } else if completed_chunks > 0 && completed_chunks < total_chunks {
        "processing" // Partial completion
    } else if total_chunks > 0 {
        "idle" // Chunks exist but not started
    } else {
        "idle"
    };

    // Terms JP progress (scoped to this paper: distinct terms with occurrences)
    let total_terms: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT term_id) FROM occurrences WHERE paper_id = ?
        "#,
    )
    .bind(&paper_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let terms_jp_status = if active_pipeline.as_deref() == Some("extract-terms-jp") {
        "processing"
    } else if paper.terms_jp_last_run_at.is_some() {
        "completed" // completed_empty is valid
    } else {
        "idle"
    };

    // Scan JP progress
    let total_occurrences = Occurrence::count_by_paper_and_method(&pool, &paper_id, "jp-scan")
        .await
        .unwrap_or(0);

    let scan_jp_status = if active_pipeline.as_deref() == Some("scan-jp") {
        "processing"
    } else if paper.scan_jp_last_run_at.is_some() {
        "completed" // completed_empty is valid
    } else {
        "idle"
    };

    // Definitions progress
    let generated_definitions: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT d.term_id) FROM definitions d
        INNER JOIN occurrences o ON d.term_id = o.term_id
        WHERE o.paper_id = ?
        "#,
    )
    .bind(&paper_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let failed_definitions = 0i64; // We don't track individual failures, only overall result_state

    let definitions_status = if active_pipeline.as_deref() == Some("generate-definitions") {
        "processing"
    } else if paper.definitions_last_run_at.is_some() {
        match paper.definitions_result_state.as_deref() {
            Some("completed_nonempty") | Some("completed_empty") => "completed",
            Some("failed") => "failed",
            _ => "idle",
        }
    } else {
        "idle"
    };

    Ok(Json(ProcessingStatusResponse {
        paper_id,
        status: paper.status.to_string(),
        translation: TranslationProgress {
            total_chunks,
            completed_chunks,
            failed_chunks,
            status: translation_status.to_string(),
        },
        terms_jp: TermsJpProgress {
            total_terms,
            last_run_at: paper.terms_jp_last_run_at.map(|t| t.to_rfc3339()),
            status: terms_jp_status.to_string(),
        },
        scan_jp: ScanJpProgress {
            total_occurrences,
            last_run_at: paper.scan_jp_last_run_at.map(|t| t.to_rfc3339()),
            status: scan_jp_status.to_string(),
        },
        definitions: DefinitionsProgress {
            generated: generated_definitions,
            failed: failed_definitions,
            last_run_at: paper.definitions_last_run_at.map(|t| t.to_rfc3339()),
            status: definitions_status.to_string(),
            result_state: paper.definitions_result_state.clone(),
        },
    }))
}
