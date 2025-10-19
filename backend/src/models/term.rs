use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

/// Term entity (data-model.md:118-154)
/// A canonical concept/vocabulary entry in the glossary
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Term {
    pub id: String,
    pub slug: String,
    pub lemma_en: String,
    pub lemma_ja: String,
    pub reading_kana: Option<String>,
    pub pos: Option<String>,
    pub tags: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Term {
    /// Create a new term
    pub async fn create(
        pool: &SqlitePool,
        slug: String,
        lemma_en: String,
        lemma_ja: String,
        reading_kana: Option<String>,
        pos: Option<String>,
        tags: Option<String>,
        note: Option<String>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query_as::<_, Term>(
            r#"
            INSERT INTO terms (id, slug, lemma_en, lemma_ja, reading_kana, pos, tags, note, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&slug)
        .bind(&lemma_en)
        .bind(&lemma_ja)
        .bind(reading_kana)
        .bind(pos)
        .bind(tags)
        .bind(note)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// Find term by ID
    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Term>(
            r#"
            SELECT * FROM terms WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Find term by slug
    pub async fn find_by_slug(pool: &SqlitePool, slug: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Term>(
            r#"
            SELECT * FROM terms WHERE slug = ?
            "#,
        )
        .bind(slug)
        .fetch_one(pool)
        .await
    }

    /// Search terms by lemma (English or Japanese)
    pub async fn search(
        pool: &SqlitePool,
        query: &str,
        lang: &str,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        match lang {
            "en" => {
                sqlx::query_as::<_, Term>(
                    r#"
                    SELECT * FROM terms
                    WHERE lemma_en LIKE ?
                    ORDER BY lemma_en
                    LIMIT ?
                    "#,
                )
                .bind(format!("%{}%", query))
                .bind(limit)
                .fetch_all(pool)
                .await
            }
            "ja" => {
                sqlx::query_as::<_, Term>(
                    r#"
                    SELECT * FROM terms
                    WHERE lemma_ja LIKE ?
                    ORDER BY lemma_ja
                    LIMIT ?
                    "#,
                )
                .bind(format!("%{}%", query))
                .bind(limit)
                .fetch_all(pool)
                .await
            }
            _ => {
                // Search both languages
                sqlx::query_as::<_, Term>(
                    r#"
                    SELECT * FROM terms
                    WHERE lemma_en LIKE ? OR lemma_ja LIKE ?
                    ORDER BY lemma_en
                    LIMIT ?
                    "#,
                )
                .bind(format!("%{}%", query))
                .bind(format!("%{}%", query))
                .bind(limit)
                .fetch_all(pool)
                .await
            }
        }
    }

    /// List all terms with pagination
    pub async fn list(
        pool: &SqlitePool,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = (page - 1) * limit;
        sqlx::query_as::<_, Term>(
            r#"
            SELECT * FROM terms
            ORDER BY lemma_en
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    /// Count total terms
    pub async fn count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM terms
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// Update term
    pub async fn update(
        pool: &SqlitePool,
        id: &str,
        lemma_en: Option<String>,
        lemma_ja: Option<String>,
        reading_kana: Option<String>,
        pos: Option<String>,
        tags: Option<String>,
        note: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let mut query = "UPDATE terms SET updated_at = ?".to_string();
        let mut params: Vec<String> = vec![Utc::now().to_rfc3339()];

        if let Some(val) = lemma_en {
            query.push_str(", lemma_en = ?");
            params.push(val);
        }
        if let Some(val) = lemma_ja {
            query.push_str(", lemma_ja = ?");
            params.push(val);
        }
        if let Some(val) = reading_kana {
            query.push_str(", reading_kana = ?");
            params.push(val);
        }
        if let Some(val) = pos {
            query.push_str(", pos = ?");
            params.push(val);
        }
        if let Some(val) = tags {
            query.push_str(", tags = ?");
            params.push(val);
        }
        if let Some(val) = note {
            query.push_str(", note = ?");
            params.push(val);
        }

        query.push_str(" WHERE id = ?");
        params.push(id.to_string());

        sqlx::query(&query)
            .bind(&params[0]) // updated_at
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Delete term (cascades to variants, definitions, occurrences)
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM terms WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
