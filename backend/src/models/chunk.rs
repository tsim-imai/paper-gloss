use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// Chunk entity (data-model.md:82-115)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Chunk {
    pub id: String,
    pub paper_id: String,
    #[sqlx(rename = "index_")]
    pub index: i32,
    pub src_text: String,
    pub trans_html: Option<String>,
    pub content_hash: String,
    pub token_count: Option<i32>,
    pub status: String,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Chunk {
    /// Create a new chunk
    pub async fn create(
        pool: &SqlitePool,
        paper_id: String,
        index: i32,
        src_text: String,
        content_hash: String,
        token_count: Option<i32>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query_as::<_, Chunk>(
            r#"
            INSERT INTO chunks (id, paper_id, index_, src_text, trans_html, content_hash, token_count, status, retry_count, error_message, created_at, updated_at)
            VALUES (?, ?, ?, ?, NULL, ?, ?, 'pending', 0, NULL, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&paper_id)
        .bind(index)
        .bind(&src_text)
        .bind(&content_hash)
        .bind(token_count)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// Find chunk by ID
    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Chunk>(
            r#"
            SELECT * FROM chunks WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Find all chunks for a paper, ordered by index
    pub async fn find_by_paper_id(pool: &SqlitePool, paper_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Chunk>(
            r#"
            SELECT * FROM chunks
            WHERE paper_id = ?
            ORDER BY index_ ASC
            "#,
        )
        .bind(paper_id)
        .fetch_all(pool)
        .await
    }

    /// Update chunk translation
    pub async fn update_translation(
        pool: &SqlitePool,
        id: &str,
        trans_html: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE chunks SET trans_html = ?, status = 'translated', updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(trans_html)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Count chunks for a paper
    pub async fn count_by_paper_id(pool: &SqlitePool, paper_id: &str) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM chunks WHERE paper_id = ?
            "#,
        )
        .bind(paper_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// Count translated chunks for a paper
    pub async fn count_translated_by_paper_id(pool: &SqlitePool, paper_id: &str) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM chunks
            WHERE paper_id = ? AND trans_html IS NOT NULL
            "#,
        )
        .bind(paper_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// Count failed chunks for a paper
    pub async fn count_failed_by_paper_id(pool: &SqlitePool, paper_id: &str) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM chunks
            WHERE paper_id = ? AND status = 'failed'
            "#,
        )
        .bind(paper_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// Delete all chunks for a paper
    pub async fn delete_by_paper_id(pool: &SqlitePool, paper_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM chunks WHERE paper_id = ?
            "#,
        )
        .bind(paper_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update chunk status and error message
    pub async fn update_status(
        pool: &SqlitePool,
        id: &str,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE chunks SET status = ?, error_message = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(error_message)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Increment retry count for a chunk
    pub async fn increment_retry(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE chunks SET retry_count = retry_count + 1, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
