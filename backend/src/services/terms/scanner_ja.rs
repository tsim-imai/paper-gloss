use crate::models::Occurrence;
use anyhow::Result;
use sqlx::SqlitePool;

/// Occurrence scanner for Japanese translated text.
pub struct OccurrenceScannerJa {
    pool: SqlitePool,
}

impl OccurrenceScannerJa {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }

    /// Scan a single chunk's Japanese text against JA variants and store occurrences.
    pub async fn scan_chunk(&self, paper_id: &str, chunk_id: &str, jp_text: &str) -> Result<usize> {
        let mut tracked = 0usize;
        // Get JA variants (longer first)
        let variants: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT term_id, surface FROM term_variants
            WHERE lang = 'ja'
            ORDER BY LENGTH(surface) DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for (term_id, surface) in variants {
            if surface.is_empty() { continue; }
            // naive substring scan (non-overlapping, left-to-right)
            let mut start = 0usize;
            while let Some(pos) = jp_text[start..].find(&surface) {
                let abs = start + pos;
                let end = abs + surface.len();
                // Convert byte offsets to char indices for robust frontend highlighting
                let start_char = jp_text[..abs].chars().count() as i32;
                let end_char = jp_text[..end].chars().count() as i32;
                // create occurrence with method 'jp-scan'
                let _ = Occurrence::create_ext(
                    &self.pool,
                    &term_id,
                    paper_id,
                    chunk_id,
                    start_char,
                    end_char,
                    Some(&surface),
                    "jp-scan",
                    None,
                ).await;
                tracked += 1;
                start = end; // non-overlapping
            }
        }
        Ok(tracked)
    }
}
