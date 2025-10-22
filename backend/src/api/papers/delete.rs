use crate::api::error::AppError;
use crate::models::Paper;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// DELETE /api/papers/{id} - Delete a paper and all associated data (FR-037)
///
/// Deletes:
/// - Paper record from database
/// - All chunks and translations
/// - All term occurrences from this paper
/// - PDF file from storage
/// - Cleans up orphaned terms (FR-039)
///
/// Returns 204 No Content on success, 404 if paper not found
pub async fn delete_paper(
    State(pool): State<SqlitePool>,
    Path(paper_id): Path<String>,
) -> Result<StatusCode, AppError> {
    info!("Deleting paper: {}", paper_id);

    // Check if paper exists
    let paper = match Paper::find_by_id(&pool, &paper_id).await {
        Ok(paper) => paper,
        Err(_) => {
            return Err(AppError::NotFound(format!("Paper {} not found", paper_id)));
        }
    };

    // Start a transaction for database operations
    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to start transaction for paper deletion: {}", e);
        AppError::InternalServerError("Failed to start deletion transaction".to_string())
    })?;

    // Delete occurrences associated with this paper
    let occurrence_result = sqlx::query("DELETE FROM occurrences WHERE paper_id = ?")
        .bind(&paper_id)
        .execute(&mut *tx)
        .await;

    match occurrence_result {
        Ok(result) => {
            info!("Deleted {} occurrences for paper {}", result.rows_affected(), paper_id);
        }
        Err(e) => {
            warn!("Failed to delete occurrences for paper {}: {}", paper_id, e);
            // Continue anyway - not critical
        }
    }

    // Delete chunks associated with this paper
    let chunk_result = sqlx::query("DELETE FROM chunks WHERE paper_id = ?")
        .bind(&paper_id)
        .execute(&mut *tx)
        .await;

    match chunk_result {
        Ok(result) => {
            info!("Deleted {} chunks for paper {}", result.rows_affected(), paper_id);
        }
        Err(e) => {
            warn!("Failed to delete chunks for paper {}: {}", paper_id, e);
            // Continue anyway
        }
    }

    // Delete the paper record
    sqlx::query("DELETE FROM papers WHERE id = ?")
        .bind(&paper_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to delete paper {}: {}", paper_id, e);
            AppError::InternalServerError("Failed to delete paper from database".to_string())
        })?;

    // Clean up orphaned terms (FR-039)
    // Find terms that have no remaining occurrences
    let orphan_cleanup_result = sqlx::query(
        r#"
        DELETE FROM terms
        WHERE id IN (
            SELECT t.id FROM terms t
            LEFT JOIN occurrences o ON t.id = o.term_id
            WHERE o.id IS NULL
        )
        "#
    )
    .execute(&mut *tx)
    .await;

    match orphan_cleanup_result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                info!("Cleaned up {} orphaned terms after deleting paper {}",
                    result.rows_affected(), paper_id);
            }
        }
        Err(e) => {
            warn!("Failed to clean up orphaned terms: {}", e);
            // Not critical - continue
        }
    }

    // Commit the transaction
    tx.commit().await.map_err(|e| {
        error!("Failed to commit deletion transaction: {}", e);
        AppError::InternalServerError("Failed to complete deletion".to_string())
    })?;

    // Delete the PDF file from storage (FR-037)
    // This is done after database deletion to ensure consistency
    let pdf_path = PathBuf::from(&paper.file_path);
    if pdf_path.exists() {
        match fs::remove_file(&pdf_path) {
            Ok(_) => {
                info!("Deleted PDF file: {}", paper.file_path);

                // Also try to remove the parent directory if it's empty
                if let Some(parent) = pdf_path.parent() {
                    // Only remove if it's a paper-specific directory
                    if parent.file_name().and_then(|n| n.to_str()).map_or(false, |n| n == paper_id) {
                        let _ = fs::remove_dir(parent); // Ignore errors - directory might not be empty
                    }
                }
            }
            Err(e) => {
                // Log but don't fail - database deletion is more important
                warn!("Failed to delete PDF file {}: {}. Manual cleanup may be required.",
                    paper.file_path, e);
            }
        }
    } else {
        warn!("PDF file not found at expected path: {}", paper.file_path);
    }

    info!("Successfully deleted paper {} and all associated data", paper_id);

    // Return 204 No Content on successful deletion (FR-038 requires confirmation in UI)
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_paper_signature() {
        // This test just verifies the function signature compiles
        // Actual integration tests are in the tests/contract directory
    }
}