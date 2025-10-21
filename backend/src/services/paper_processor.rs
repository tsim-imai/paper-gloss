use crate::models::{Chunk, Paper, PaperStatus, Term};
use crate::services::llm::{LlmClient, LlmLogger};
use crate::services::pdf::{PdfExtractor, TextChunker};
use crate::services::terms::{TermExtractor, OccurrenceTracker, DefinitionGenerator};
use crate::services::translation::TranslationService;
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::path::Path;
use tracing::{debug, info, warn};

/// Paper processing orchestrator (FR-031, FR-032, FR-033)
/// Coordinates: extraction → chunking → translation → term extraction
pub struct PaperProcessor {
    pool: SqlitePool,
    translation_service: TranslationService,
    llm_client: LlmClient,
}

impl PaperProcessor {
    pub fn new(pool: SqlitePool) -> Result<Self> {
        Ok(Self {
            pool: pool.clone(),
            translation_service: TranslationService::new()?,
            llm_client: LlmClient::new()?,
        })
    }

    /// Process a paper: extract, chunk, and translate
    pub async fn process_paper(&self, paper_id: &str) -> Result<()> {
        info!("Starting paper processing: {}", paper_id);

        // Update status to processing
        Paper::update_status(&self.pool, paper_id, PaperStatus::Processing).await?;

        // Step 1: Find paper
        let paper = Paper::find_by_id(&self.pool, paper_id)
            .await
            .context("Failed to find paper")?;

        // Step 2: Extract text from PDF
        let file_path = Path::new(&paper.file_path);
        let extraction_result = match PdfExtractor::extract_with_recovery(file_path) {
            Ok(result) => result,
            Err(e) => {
                warn!("PDF extraction failed for {}: {}", paper_id, e);
                Paper::update_status(&self.pool, paper_id, PaperStatus::Failed).await?;
                return Err(e);
            }
        };

        if !extraction_result.failed_pages.is_empty() {
            warn!(
                "PDF extraction had {} failed pages",
                extraction_result.failed_pages.len()
            );
        }

        // Step 3: Chunk text
        let chunker = TextChunker::default();
        let chunks = chunker.chunk(&extraction_result.text);

        info!("Created {} chunks for paper {}", chunks.len(), paper_id);

        // Step 4: Save chunks to database
        for chunk in &chunks {
            Chunk::create(
                &self.pool,
                paper_id.to_string(),
                chunk.index as i32,
                chunk.text.clone(),
                chunk.content_hash.clone(),
                chunk.token_count.map(|t| t as i32),
            )
            .await
            .context("Failed to create chunk")?;
        }

        // Step 5: Translate chunks
        let llm_logger = LlmLogger::new(paper_id)?;

        for chunk in &chunks {
            match self.translation_service.translate_chunk(&chunk.text, 3).await {
                Ok(result) => {
                    // Find chunk by index
                    let db_chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;
                    if let Some(db_chunk) = db_chunks.get(chunk.index) {
                        Chunk::update_translation(
                            &self.pool,
                            &db_chunk.id,
                            result.translated_text.clone(),
                        )
                        .await?;

                        // Log successful translation
                        llm_logger
                            .log_translation(
                                &db_chunk.id,
                                &chunk.text,
                                Some(&result.translated_text),
                                None,
                                result.duration.as_millis(),
                            )
                            .ok();

                        debug!("Translated chunk {} for paper {}", chunk.index, paper_id);
                    }
                }
                Err(e) => {
                    warn!("Translation failed for chunk {}: {}", chunk.index, e);

                    // Update chunk status to failed
                    let db_chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;
                    if let Some(db_chunk) = db_chunks.get(chunk.index) {
                        Chunk::update_status(&self.pool, &db_chunk.id, "failed", Some(e.to_string())).await.ok();
                    }

                    // Log failed translation
                    llm_logger
                        .log_translation(
                            &format!("chunk_{}", chunk.index),
                            &chunk.text,
                            None,
                            Some(e.to_string()),
                            0,
                        )
                        .ok();

                    // FR-033: Preserve partial results - continue processing other chunks
                    continue;
                }
            }
        }

        // Check if any chunks were successfully translated
        let translated_count = Chunk::count_translated_by_paper_id(&self.pool, paper_id).await?;
        let total_count = Chunk::count_by_paper_id(&self.pool, paper_id).await?;

        if translated_count == 0 {
            warn!("No chunks were successfully translated for paper {}", paper_id);
            Paper::update_status(&self.pool, paper_id, PaperStatus::Failed).await?;
            return Ok(());
        }

        // Step 6: Extract terms from source text (FR-017)
        info!("Extracting terms from paper {}", paper_id);
        if let Err(e) = self.extract_terms(paper_id).await {
            warn!("Term extraction failed for paper {}: {}", paper_id, e);
            // Continue anyway - translation is more critical
        }

        // Step 7: Track term occurrences in translated text
        info!("Tracking term occurrences for paper {}", paper_id);
        if let Err(e) = self.track_occurrences(paper_id).await {
            warn!("Occurrence tracking failed for paper {}: {}", paper_id, e);
            // Continue anyway
        }

        // Step 8: Generate definitions for extracted terms
        info!("Generating definitions for paper {}", paper_id);
        if let Err(e) = self.generate_definitions(paper_id).await {
            warn!("Definition generation failed for paper {}: {}", paper_id, e);
            // Continue anyway - definitions can be generated later
        }

        // Update final status based on translation completion
        if translated_count < total_count {
            warn!(
                "Partial translation: {}/{} chunks for paper {}",
                translated_count, total_count, paper_id
            );
            // Keep status as Processing (partial completion)
            // User can retry failed chunks
        } else {
            info!("Successfully processed paper {}", paper_id);
            Paper::update_status(&self.pool, paper_id, PaperStatus::Completed).await?;
        }

        Ok(())
    }

    /// Extract terms from paper source text
    async fn extract_terms(&self, paper_id: &str) -> Result<()> {
        let term_extractor = TermExtractor::new(self.llm_client.clone(), self.pool.clone());

        let chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;

        // Extract terms from each chunk's source text
        for chunk in chunks {
            // Only extract from chunks with translations
            if chunk.trans_html.is_some() {
                match term_extractor.extract_terms(&chunk.src_text).await {
                    Ok(extracted_terms) => {
                        term_extractor
                            .store_terms(paper_id, &chunk.id, extracted_terms)
                            .await?;
                        debug!("Extracted terms from chunk {}", chunk.id);
                    }
                    Err(e) => {
                        warn!("Failed to extract terms from chunk {}: {}", chunk.id, e);
                        // Continue with other chunks
                    }
                }
            }
        }

        Ok(())
    }

    /// Track term occurrences in translated chunks
    async fn track_occurrences(&self, paper_id: &str) -> Result<()> {
        let tracker = OccurrenceTracker::new(self.pool.clone());

        let chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;

        for chunk in chunks {
            if let Some(trans_html) = &chunk.trans_html {
                match tracker.track_occurrences(paper_id, &chunk.id, trans_html).await {
                    Ok(count) => {
                        debug!("Tracked {} occurrences in chunk {}", count, chunk.id);
                    }
                    Err(e) => {
                        warn!("Failed to track occurrences in chunk {}: {}", chunk.id, e);
                        // Continue with other chunks
                    }
                }
            }
        }

        Ok(())
    }

    /// Retry translation for a specific chunk (FR-032)
    pub async fn retry_chunk(&self, chunk_id: &str) -> Result<()> {
        info!("Retrying chunk translation: {}", chunk_id);

        let chunk = Chunk::find_by_id(&self.pool, chunk_id)
            .await
            .context("Failed to find chunk")?;

        // Increment retry count
        Chunk::increment_retry(&self.pool, chunk_id).await?;

        let llm_logger = LlmLogger::new(&chunk.paper_id)?;

        match self.translation_service.translate_chunk(&chunk.src_text, 3).await {
            Ok(result) => {
                Chunk::update_translation(&self.pool, chunk_id, result.translated_text.clone())
                    .await?;

                llm_logger
                    .log_translation(
                        chunk_id,
                        &chunk.src_text,
                        Some(&result.translated_text),
                        None,
                        result.duration.as_millis(),
                    )
                    .ok();

                info!("Successfully retried chunk {}", chunk_id);
                Ok(())
            }
            Err(e) => {
                warn!("Chunk retry failed for {}: {}", chunk_id, e);

                // Update status to failed with error message
                Chunk::update_status(&self.pool, chunk_id, "failed", Some(e.to_string())).await?;

                llm_logger
                    .log_translation(chunk_id, &chunk.src_text, None, Some(e.to_string()), 0)
                    .ok();

                Err(e)
            }
        }
    }

    /// Generate definitions for terms without definitions
    async fn generate_definitions(&self, paper_id: &str) -> Result<()> {
        let def_generator = DefinitionGenerator::new(self.llm_client.clone(), self.pool.clone());

        // Get all terms from occurrences in this paper
        let term_ids: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT term_id FROM occurrences
            WHERE paper_id = ?
            "#,
        )
        .bind(paper_id)
        .fetch_all(&self.pool)
        .await?;

        for term_id in term_ids {
            // Check if definition already exists
            match crate::models::Definition::find_by_term_id(&self.pool, &term_id).await {
                Ok(Some(_)) => {
                    // Definition already exists, skip
                    debug!("Definition already exists for term {}", term_id);
                    continue;
                }
                Ok(None) => {
                    // No definition, generate one
                    match Term::find_by_id(&self.pool, &term_id).await {
                        Ok(term) => {
                            match def_generator
                                .generate_and_store(&term.id, &term.lemma_en, &term.lemma_ja, None)
                                .await
                            {
                                Ok(_) => {
                                    info!("Generated definition for term {}", term.lemma_en);
                                }
                                Err(e) => {
                                    warn!("Failed to generate definition for {}: {}", term.lemma_en, e);
                                    // Continue with other terms
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to find term {}: {}", term_id, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to check definition for term {}: {}", term_id, e);
                }
            }
        }

        Ok(())
    }
}
