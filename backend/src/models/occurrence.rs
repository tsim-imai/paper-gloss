use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// Occurrence entity (data-model.md:220-254)
/// A specific instance where a term appears in a translated paper
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Occurrence {
    pub id: String,
    pub term_id: String,
    pub paper_id: String,
    pub chunk_id: String,
    pub start_pos: i32,
    pub end_pos: i32,
    pub created_at: DateTime<Utc>,
    // New fields (003 migration)
    pub surface: Option<String>,     // Japanese text that was matched
    pub method: String,               // 'jp-scan' for JP-first pipeline
    pub variant_id: Option<String>,  // Link to term_variant if exists
}

impl Occurrence {
    /// Create a new occurrence
    pub async fn create(
        pool: &SqlitePool,
        term_id: String,
        paper_id: String,
        chunk_id: String,
        start_pos: i32,
        end_pos: i32,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query_as::<_, Occurrence>(
            r#"
            INSERT INTO occurrences (id, term_id, paper_id, chunk_id, start_pos, end_pos, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&term_id)
        .bind(&paper_id)
        .bind(&chunk_id)
        .bind(start_pos)
        .bind(end_pos)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// Create a new occurrence with extended fields (surface/method/variant_id)
    pub async fn create_ext(
        pool: &SqlitePool,
        term_id: &str,
        paper_id: &str,
        chunk_id: &str,
        start_pos: i32,
        end_pos: i32,
        surface: Option<&str>,
        method: &str,
        variant_id: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query_as::<_, Occurrence>(
            r#"
            INSERT INTO occurrences (id, term_id, paper_id, chunk_id, start_pos, end_pos, created_at, surface, method, variant_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(term_id)
        .bind(paper_id)
        .bind(chunk_id)
        .bind(start_pos)
        .bind(end_pos)
        .bind(now)
        .bind(surface)
        .bind(method)
        .bind(variant_id)
        .fetch_one(pool)
        .await
    }

    /// Find all occurrences for a term
    pub async fn find_by_term_id(
        pool: &SqlitePool,
        term_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Occurrence>(
            r#"
            SELECT * FROM occurrences
            WHERE term_id = ?
            ORDER BY paper_id, chunk_id, start_pos
            "#,
        )
        .bind(term_id)
        .fetch_all(pool)
        .await
    }

    /// Find all occurrences in a paper
    pub async fn find_by_paper_id(
        pool: &SqlitePool,
        paper_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Occurrence>(
            r#"
            SELECT * FROM occurrences
            WHERE paper_id = ?
            ORDER BY chunk_id, start_pos
            "#,
        )
        .bind(paper_id)
        .fetch_all(pool)
        .await
    }

    /// Find all occurrences in a chunk
    pub async fn find_by_chunk_id(
        pool: &SqlitePool,
        chunk_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Occurrence>(
            r#"
            SELECT * FROM occurrences
            WHERE chunk_id = ?
            ORDER BY start_pos
            "#,
        )
        .bind(chunk_id)
        .fetch_all(pool)
        .await
    }

    /// Count occurrences for a term
    pub async fn count_by_term_id(pool: &SqlitePool, term_id: &str) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM occurrences WHERE term_id = ?
            "#,
        )
        .bind(term_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// Delete occurrence
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM occurrences WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete all occurrences for a paper
    pub async fn delete_by_paper_id(pool: &SqlitePool, paper_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM occurrences WHERE paper_id = ?
            "#,
        )
        .bind(paper_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete all occurrences for a paper with a specific method (for re-run)
    pub async fn delete_by_paper_and_method(
        pool: &SqlitePool,
        paper_id: &str,
        method: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM occurrences WHERE paper_id = ? AND method = ?
            "#,
        )
        .bind(paper_id)
        .bind(method)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Count occurrences for a paper with a specific method
    pub async fn count_by_paper_and_method(
        pool: &SqlitePool,
        paper_id: &str,
        method: &str,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM occurrences WHERE paper_id = ? AND method = ?
            "#,
        )
        .bind(paper_id)
        .bind(method)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }
}
