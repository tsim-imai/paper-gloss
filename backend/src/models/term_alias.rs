use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TermAlias {
    pub id: String,
    pub term_id: String,
    pub surface: String,
    pub lang: String,
    pub kind: String,
    pub confidence: Option<f64>,
    pub created_at: DateTime<Utc>,
}

impl TermAlias {
    pub async fn create(
        pool: &SqlitePool,
        term_id: String,
        surface: String,
        lang: String,
        kind: String,
        confidence: Option<f64>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        sqlx::query_as::<_, TermAlias>(
            r#"
            INSERT INTO term_aliases (id, term_id, surface, lang, kind, confidence, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&term_id)
        .bind(&surface)
        .bind(&lang)
        .bind(&kind)
        .bind(confidence)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// Find all aliases for a term
    pub async fn find_by_term_id(
        pool: &SqlitePool,
        term_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, TermAlias>(
            r#"
            SELECT * FROM term_aliases
            WHERE term_id = ?
            ORDER BY lang, kind, surface
            "#,
        )
        .bind(term_id)
        .fetch_all(pool)
        .await
    }
}
