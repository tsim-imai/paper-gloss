use crate::models::{Term, TermVariant};
use crate::services::llm::LlmClient;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Extracted term from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTerm {
    pub lemma_en: String,
    pub lemma_ja: String,
    pub reading_kana: Option<String>,
    pub pos: Option<String>,
    pub category: Option<String>,
}

/// Term extraction service using LLM (spec.md:FR-017)
#[derive(Clone)]
pub struct TermExtractor {
    llm_client: LlmClient,
    pool: SqlitePool,
}

impl TermExtractor {
    pub fn new(llm_client: LlmClient, pool: SqlitePool) -> Self {
        Self { llm_client, pool }
    }

    /// Extract terms from English text using LLM
    pub async fn extract_terms(&self, text: &str) -> Result<Vec<ExtractedTerm>> {
        let prompt = format!(
            r#"Extract machine learning and AI technical terms from the following text.
For each term, provide:
1. English lemma (canonical form)
2. Japanese translation (katakana/kanji)
3. Reading in hiragana (optional)
4. Part of speech (noun/verb/adjective)
5. Category tags (comma-separated, e.g., "deep-learning,nlp")

Return as JSON array with fields: lemma_en, lemma_ja, reading_kana, pos, category

Text:
{}

Return ONLY valid JSON array, no additional text."#,
            text
        );

        use crate::services::llm::Message;
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are an expert in machine learning terminology extraction.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let response = self.llm_client.chat_completion(messages, None, Some(2000)).await
            .context("Failed to extract terms from LLM")?;

        // Parse JSON response
        let terms: Vec<ExtractedTerm> = serde_json::from_str(&response)
            .context("Failed to parse term extraction response")?;

        Ok(terms)
    }

    /// Store extracted terms in database with variants
    /// Note: Occurrences are tracked separately in translated text by OccurrenceTracker
    pub async fn store_terms(
        &self,
        _paper_id: &str,
        _chunk_id: &str,
        extracted_terms: Vec<ExtractedTerm>,
    ) -> Result<Vec<String>> {
        let mut term_ids = Vec::new();

        for extracted in extracted_terms {
            // Generate slug from English lemma
            let slug = extracted.lemma_en
                .to_lowercase()
                .replace(' ', "-")
                .replace('_', "-");

            // Check if term already exists by slug
            let existing_term = Term::find_by_slug(&self.pool, &slug).await.ok();

            let term_id = if let Some(term) = existing_term {
                // Term already exists, use existing ID
                term.id
            } else {
                // Create new term
                let tags = extracted.category.clone();
                let term = Term::create(
                    &self.pool,
                    slug.clone(),
                    extracted.lemma_en.clone(),
                    extracted.lemma_ja.clone(),
                    extracted.reading_kana.clone(),
                    extracted.pos.clone(),
                    tags,
                    None,
                )
                .await
                .context("Failed to create term")?;

                // Create variants for English lemma
                TermVariant::create(
                    &self.pool,
                    term.id.clone(),
                    "en".to_string(),
                    extracted.lemma_en.clone(),
                )
                .await
                .ok(); // Ignore if variant already exists

                // Create variants for Japanese lemma
                TermVariant::create(
                    &self.pool,
                    term.id.clone(),
                    "ja".to_string(),
                    extracted.lemma_ja.clone(),
                )
                .await
                .ok(); // Ignore if variant already exists

                term.id
            };

            term_ids.push(term_id);
        }

        Ok(term_ids)
    }
}
