use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// Definition entity (data-model.md:189-217)
/// A human-readable explanation of a term in Japanese
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Definition {
    pub id: String,
    pub term_id: String,
    pub lang: String,
    pub text: String,
    pub provider: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Definition {
    /// Create a new definition
    pub async fn create(
        pool: &SqlitePool,
        term_id: String,
        text: String,
        provider: String,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query_as::<_, Definition>(
            r#"
            INSERT INTO definitions (id, term_id, lang, text, provider, created_at, updated_at)
            VALUES (?, ?, 'ja', ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&term_id)
        .bind(&text)
        .bind(&provider)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// Find definition by term ID
    pub async fn find_by_term_id(
        pool: &SqlitePool,
        term_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Definition>(
            r#"
            SELECT * FROM definitions WHERE term_id = ?
            "#,
        )
        .bind(term_id)
        .fetch_optional(pool)
        .await
    }

    /// Update definition text
    pub async fn update(
        pool: &SqlitePool,
        term_id: &str,
        text: String,
        provider: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE definitions SET text = ?, provider = ?, updated_at = ?
            WHERE term_id = ?
            "#,
        )
        .bind(text)
        .bind(provider)
        .bind(Utc::now())
        .bind(term_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete definition
    pub async fn delete(pool: &SqlitePool, term_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM definitions WHERE term_id = ?
            "#,
        )
        .bind(term_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
