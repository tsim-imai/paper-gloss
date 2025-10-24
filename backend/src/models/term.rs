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
        note: Option<String>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query_as::<_, Term>(
            r#"
            INSERT INTO terms (id, slug, lemma_en, lemma_ja, reading_kana, note, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&slug)
        .bind(&lemma_en)
        .bind(&lemma_ja)
        .bind(reading_kana)
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
    /// Also searches in term_variants for better matching
    pub async fn search(
        pool: &SqlitePool,
        query: &str,
        lang: &str,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        // Use LIKE pattern for normalized query
        let like_pattern = format!("%{}%", query);

        match lang {
            "en" => {
                // Search in both lemma and term_variants
                sqlx::query_as::<_, Term>(
                    r#"
                    SELECT DISTINCT t.* FROM terms t
                    LEFT JOIN term_variants tv ON tv.term_id = t.id
                    WHERE t.lemma_en LIKE ?
                       OR (tv.lang = 'en' AND tv.surface LIKE ?)
                    ORDER BY t.lemma_en
                    LIMIT ?
                    "#,
                )
                .bind(&like_pattern)
                .bind(&like_pattern)
                .bind(limit)
                .fetch_all(pool)
                .await
            }
            "ja" => {
                // Search in both lemma and term_variants
                sqlx::query_as::<_, Term>(
                    r#"
                    SELECT DISTINCT t.* FROM terms t
                    LEFT JOIN term_variants tv ON tv.term_id = t.id
                    WHERE t.lemma_ja LIKE ?
                       OR (tv.lang = 'ja' AND tv.surface LIKE ?)
                    ORDER BY t.lemma_ja
                    LIMIT ?
                    "#,
                )
                .bind(&like_pattern)
                .bind(&like_pattern)
                .bind(limit)
                .fetch_all(pool)
                .await
            }
            _ => {
                // Search both languages and all variants
                sqlx::query_as::<_, Term>(
                    r#"
                    SELECT DISTINCT t.* FROM terms t
                    LEFT JOIN term_variants tv ON tv.term_id = t.id
                    WHERE t.lemma_en LIKE ?
                       OR t.lemma_ja LIKE ?
                       OR tv.surface LIKE ?
                    ORDER BY t.lemma_en
                    LIMIT ?
                    "#,
                )
                .bind(&like_pattern)
                .bind(&like_pattern)
                .bind(&like_pattern)
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

    /// List all terms (no pagination) for global sorting cases
    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Term>(
            r#"
            SELECT * FROM terms
            ORDER BY lemma_en
            "#,
        )
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
        note: Option<String>,
    ) -> Result<(), sqlx::Error> {
        // COALESCE-based static update to avoid dynamic SQL binding issues
        sqlx::query(
            r#"
            UPDATE terms SET
                lemma_en = COALESCE(?, lemma_en),
                lemma_ja = COALESCE(?, lemma_ja),
                reading_kana = COALESCE(?, reading_kana),
                note = COALESCE(?, note),
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(lemma_en)
        .bind(lemma_ja)
        .bind(reading_kana)
        .bind(note)
        .bind(Utc::now())
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
