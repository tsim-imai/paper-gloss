use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DefinitionMeta {
    pub id: String,
    pub term_id: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub prompt_version: Option<String>,
    pub confidence: Option<f64>,
    pub flags: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl DefinitionMeta {
    pub async fn find_by_term_id(
        pool: &SqlitePool,
        term_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, DefinitionMeta>(
            r#"
            SELECT * FROM definition_meta WHERE term_id = ?
            "#,
        )
        .bind(term_id)
        .fetch_optional(pool)
        .await
    }
}

