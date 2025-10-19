use crate::models::{Occurrence, TermVariant};
use anyhow::Result;
use sqlx::SqlitePool;

/// Occurrence tracking service (data-model.md:220-254)
pub struct OccurrenceTracker {
    pool: SqlitePool,
}

impl OccurrenceTracker {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Track all term occurrences in a chunk's translated text
    pub async fn track_occurrences(
        &self,
        paper_id: &str,
        chunk_id: &str,
        text: &str,
    ) -> Result<usize> {
        let mut tracked_count = 0;

        // Get all term variants to search for
        let all_variants = self.get_all_variants().await?;

        for (term_id, surface, lang) in all_variants {
            // Only track in translated (Japanese) text for now
            if lang == "ja" {
                let occurrences = self.find_surface_occurrences(&surface, text);

                for (start_pos, end_pos) in occurrences {
                    // Store occurrence (ignore duplicates)
                    match Occurrence::create(
                        &self.pool,
                        term_id.clone(),
                        paper_id.to_string(),
                        chunk_id.to_string(),
                        start_pos as i32,
                        end_pos as i32,
                    )
                    .await
                    {
                        Ok(_) => tracked_count += 1,
                        Err(_) => {} // Ignore duplicate constraint errors
                    }
                }
            }
        }

        Ok(tracked_count)
    }

    /// Get all term variants for matching
    async fn get_all_variants(&self) -> Result<Vec<(String, String, String)>> {
        let variants: Vec<(String, String, String)> = sqlx::query_as(
            r#"
            SELECT term_id, surface, lang
            FROM term_variants
            ORDER BY LENGTH(surface) DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(variants)
    }

    /// Find all occurrences of a surface form in text
    fn find_surface_occurrences(&self, surface: &str, text: &str) -> Vec<(usize, usize)> {
        let mut occurrences = Vec::new();
        let mut start = 0;

        while let Some(pos) = text[start..].find(surface) {
            let absolute_pos = start + pos;
            occurrences.push((absolute_pos, absolute_pos + surface.len()));
            start = absolute_pos + 1;
        }

        occurrences
    }

    /// Delete all occurrences for a paper (cleanup)
    pub async fn clear_paper_occurrences(&self, paper_id: &str) -> Result<()> {
        Occurrence::delete_by_paper_id(&self.pool, paper_id).await?;
        Ok(())
    }
}
