use crate::models::{Occurrence, Term};
use crate::services::terms::normalization::{normalize_english, normalize_japanese};
use anyhow::Result;
use sqlx::SqlitePool;

/// Search results with occurrence counts
#[derive(Debug, Clone)]
pub struct TermSearchResult {
    pub term: Term,
    pub occurrence_count: i64,
}

/// Term search service with bilingual support and normalization
pub struct TermSearchService {
    pool: SqlitePool,
}

impl TermSearchService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Search terms with bilingual support
    ///
    /// # Arguments
    /// * `query` - Search query (can be English or Japanese)
    /// * `lang` - Language filter: "en", "ja", or "both"
    /// * `sort` - Sort order: "alphabetical" or "frequency"
    /// * `page` - Page number (1-indexed)
    /// * `limit` - Results per page
    pub async fn search(
        &self,
        query: Option<&str>,
        lang: &str,
        sort: &str,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<TermSearchResult>, i64)> {
        // If no query, just list all terms
        if query.is_none() || query == Some("") {
            return self.list_all(sort, page, limit).await;
        }

        let query_str = query.unwrap();
        let normalized_query = self.normalize_query(query_str, lang);

        // Search terms based on language
        let terms = match lang {
            "en" => self.search_english(&normalized_query, limit).await?,
            "ja" => self.search_japanese(&normalized_query, limit).await?,
            _ => self.search_both(&normalized_query, limit).await?,
        };

        // Get occurrence counts for sorting
        let mut results = Vec::new();
        for term in terms {
            let count = Occurrence::count_by_term_id(&self.pool, &term.id).await?;
            results.push(TermSearchResult {
                term,
                occurrence_count: count,
            });
        }

        // Sort results
        self.sort_results(&mut results, sort);

        // Paginate
        let total = results.len() as i64;
        let offset = ((page - 1) * limit) as usize;
        let end = ((page * limit) as usize).min(results.len());
        let paginated = if offset < results.len() {
            results[offset..end].to_vec()
        } else {
            Vec::new()
        };

        Ok((paginated, total))
    }

    /// List all terms without search filter
    async fn list_all(
        &self,
        sort: &str,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<TermSearchResult>, i64)> {
        let total = Term::count(&self.pool).await?;
        let terms = Term::list(&self.pool, page, limit).await?;

        let mut results = Vec::new();
        for term in terms {
            let count = Occurrence::count_by_term_id(&self.pool, &term.id).await?;
            results.push(TermSearchResult {
                term,
                occurrence_count: count,
            });
        }

        self.sort_results(&mut results, sort);

        Ok((results, total))
    }

    /// Normalize query based on language
    fn normalize_query(&self, query: &str, lang: &str) -> String {
        match lang {
            "en" => normalize_english(query),
            "ja" => normalize_japanese(query),
            _ => {
                // For "both", try to detect language and normalize accordingly
                if query.chars().any(|c| c as u32 > 0x3000) {
                    // Contains CJK characters
                    normalize_japanese(query)
                } else {
                    normalize_english(query)
                }
            }
        }
    }

    /// Search English terms
    async fn search_english(&self, normalized_query: &str, limit: i64) -> Result<Vec<Term>> {
        let terms = Term::search(&self.pool, normalized_query, "en", limit).await?;
        Ok(terms)
    }

    /// Search Japanese terms
    async fn search_japanese(&self, normalized_query: &str, limit: i64) -> Result<Vec<Term>> {
        let terms = Term::search(&self.pool, normalized_query, "ja", limit).await?;
        Ok(terms)
    }

    /// Search both English and Japanese terms
    async fn search_both(&self, normalized_query: &str, limit: i64) -> Result<Vec<Term>> {
        let terms = Term::search(&self.pool, normalized_query, "both", limit).await?;
        Ok(terms)
    }

    /// Sort search results
    fn sort_results(&self, results: &mut [TermSearchResult], sort: &str) {
        match sort {
            "frequency" => {
                results.sort_by(|a, b| {
                    b.occurrence_count
                        .cmp(&a.occurrence_count)
                        .then_with(|| a.term.lemma_en.cmp(&b.term.lemma_en))
                });
            }
            _ => {
                // Default: alphabetical by English lemma
                results.sort_by(|a, b| a.term.lemma_en.cmp(&b.term.lemma_en));
            }
        }
    }
}
