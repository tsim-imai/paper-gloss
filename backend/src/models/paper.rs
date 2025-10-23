use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// Paper entity (data-model.md:44-78)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub source_url: Option<String>,
    pub file_path: String,
    pub status: PaperStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Pipeline execution tracking (migration 004)
    pub translation_last_run_at: Option<DateTime<Utc>>,
    pub terms_jp_last_run_at: Option<DateTime<Utc>>,
    pub scan_jp_last_run_at: Option<DateTime<Utc>>,
    pub definitions_last_run_at: Option<DateTime<Utc>>,
    pub definitions_result_state: Option<String>,
    // Terms extraction count (migration 006)
    pub terms_jp_extracted_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum PaperStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

impl std::fmt::Display for PaperStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaperStatus::Pending => write!(f, "pending"),
            PaperStatus::Processing => write!(f, "processing"),
            PaperStatus::Completed => write!(f, "completed"),
            PaperStatus::Failed => write!(f, "failed"),
        }
    }
}

impl Paper {
    /// Create a new paper
    pub async fn create(
        pool: &SqlitePool,
        title: String,
        source_url: Option<String>,
        file_path: String,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let status = PaperStatus::Pending;
        let now = Utc::now();

        sqlx::query_as::<_, Paper>(
            r#"
            INSERT INTO papers (id, title, source_url, file_path, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&title)
        .bind(&source_url)
        .bind(&file_path)
        .bind(status.to_string())
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// Find paper by ID
    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Paper>(
            r#"
            SELECT * FROM papers WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// List all papers with optional status filter
    pub async fn list(
        pool: &SqlitePool,
        status: Option<PaperStatus>,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = (page - 1) * limit;

        let query = match status {
            Some(s) => sqlx::query_as::<_, Paper>(
                r#"
                SELECT * FROM papers
                WHERE status = ?
                ORDER BY created_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(s.to_string())
            .bind(limit)
            .bind(offset),
            None => sqlx::query_as::<_, Paper>(
                r#"
                SELECT * FROM papers
                ORDER BY created_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(limit)
            .bind(offset),
        };

        query.fetch_all(pool).await
    }

    /// Count papers with optional status filter
    pub async fn count(pool: &SqlitePool, status: Option<PaperStatus>) -> Result<i64, sqlx::Error> {
        let count: (i64,) = match status {
            Some(s) => sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM papers WHERE status = ?
                "#,
            )
            .bind(s.to_string())
            .fetch_one(pool)
            .await?,
            None => sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM papers
                "#,
            )
            .fetch_one(pool)
            .await?,
        };

        Ok(count.0)
    }

    /// Update paper status
    pub async fn update_status(
        pool: &SqlitePool,
        id: &str,
        status: PaperStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE papers SET status = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status.to_string())
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete paper by ID
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM papers WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update translation pipeline timestamp
    pub async fn update_translation_run_at(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE papers SET translation_last_run_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update terms_jp pipeline timestamp and extracted count
    pub async fn update_terms_jp_run_at(pool: &SqlitePool, id: &str, extracted_count: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE papers SET terms_jp_last_run_at = ?, terms_jp_extracted_count = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(Utc::now())
        .bind(extracted_count)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update scan_jp pipeline timestamp
    pub async fn update_scan_jp_run_at(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE papers SET scan_jp_last_run_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update definitions pipeline timestamp and result state
    pub async fn update_definitions_run_at(
        pool: &SqlitePool,
        id: &str,
        result_state: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE papers SET definitions_last_run_at = ?, definitions_result_state = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(Utc::now())
        .bind(result_state)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
