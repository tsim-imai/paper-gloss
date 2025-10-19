use crate::models::Term;
use crate::services::terms::normalization::{normalize_english, normalize_japanese};
use anyhow::Result;
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Potential duplicate term pair
#[derive(Debug, Clone)]
pub struct DuplicatePair {
    pub term1: Term,
    pub term2: Term,
    pub similarity_score: f64,
    pub reason: String,
}

/// Duplicate detection service
pub struct DuplicateDetectionService {
    pool: SqlitePool,
}

impl DuplicateDetectionService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find potential duplicate terms
    ///
    /// Detects duplicates based on:
    /// - Exact normalized match (English or Japanese)
    /// - Similar English lemmas (case-insensitive, hyphen/space variants)
    /// - Similar Japanese lemmas (katakana normalization)
    pub async fn find_duplicates(&self) -> Result<Vec<DuplicatePair>> {
        let terms = Term::list(&self.pool, 1, 10000).await?; // Get all terms

        let mut duplicates = Vec::new();

        // Group terms by normalized English lemma
        let mut en_groups: HashMap<String, Vec<Term>> = HashMap::new();
        for term in &terms {
            let normalized = normalize_english(&term.lemma_en);
            en_groups.entry(normalized).or_default().push(term.clone());
        }

        // Find English duplicates
        for (normalized, group) in &en_groups {
            if group.len() > 1 {
                for i in 0..group.len() {
                    for j in (i + 1)..group.len() {
                        duplicates.push(DuplicatePair {
                            term1: group[i].clone(),
                            term2: group[j].clone(),
                            similarity_score: 1.0,
                            reason: format!("Exact normalized English match: '{}'", normalized),
                        });
                    }
                }
            }
        }

        // Group terms by normalized Japanese lemma
        let mut ja_groups: HashMap<String, Vec<Term>> = HashMap::new();
        for term in &terms {
            let normalized = normalize_japanese(&term.lemma_ja);
            ja_groups.entry(normalized).or_default().push(term.clone());
        }

        // Find Japanese duplicates
        for (normalized, group) in &ja_groups {
            if group.len() > 1 {
                for i in 0..group.len() {
                    for j in (i + 1)..group.len() {
                        // Skip if already found via English match
                        let already_found = duplicates.iter().any(|d| {
                            (d.term1.id == group[i].id && d.term2.id == group[j].id)
                                || (d.term1.id == group[j].id && d.term2.id == group[i].id)
                        });

                        if !already_found {
                            duplicates.push(DuplicatePair {
                                term1: group[i].clone(),
                                term2: group[j].clone(),
                                similarity_score: 1.0,
                                reason: format!("Exact normalized Japanese match: '{}'", normalized),
                            });
                        }
                    }
                }
            }
        }

        // Sort by similarity score (highest first)
        duplicates.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(duplicates)
    }

    /// Check if two specific terms are duplicates
    pub async fn are_duplicates(&self, term1_id: &str, term2_id: &str) -> Result<bool> {
        let term1 = Term::find_by_id(&self.pool, term1_id).await?;
        let term2 = Term::find_by_id(&self.pool, term2_id).await?;

        // Check normalized English
        let en1 = normalize_english(&term1.lemma_en);
        let en2 = normalize_english(&term2.lemma_en);
        if en1 == en2 {
            return Ok(true);
        }

        // Check normalized Japanese
        let ja1 = normalize_japanese(&term1.lemma_ja);
        let ja2 = normalize_japanese(&term2.lemma_ja);
        if ja1 == ja2 {
            return Ok(true);
        }

        Ok(false)
    }
}
