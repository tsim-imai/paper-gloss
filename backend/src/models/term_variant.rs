use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// TermVariant entity (data-model.md:158-186)
/// A spelling or expression variation of a canonical term
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TermVariant {
    pub id: String,
    pub term_id: String,
    pub lang: String,
    pub surface: String,
    pub created_at: DateTime<Utc>,
}

impl TermVariant {
    /// Create a new term variant
    pub async fn create(
        pool: &SqlitePool,
        term_id: String,
        lang: String,
        surface: String,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query_as::<_, TermVariant>(
            r#"
            INSERT INTO term_variants (id, term_id, lang, surface, created_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&term_id)
        .bind(&lang)
        .bind(&surface)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// Find all variants for a term
    pub async fn find_by_term_id(
        pool: &SqlitePool,
        term_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, TermVariant>(
            r#"
            SELECT * FROM term_variants
            WHERE term_id = ?
            ORDER BY lang, surface
            "#,
        )
        .bind(term_id)
        .fetch_all(pool)
        .await
    }

    /// Find term ID by surface form
    pub async fn find_term_by_surface(
        pool: &SqlitePool,
        surface: &str,
        lang: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let result: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT term_id FROM term_variants
            WHERE surface = ? AND lang = ?
            LIMIT 1
            "#,
        )
        .bind(surface)
        .bind(lang)
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|r| r.0))
    }

    /// Delete variant
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM term_variants WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
