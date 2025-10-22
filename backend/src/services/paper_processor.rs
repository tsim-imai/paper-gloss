use crate::models::{Chunk, Paper, PaperStatus, Term};
use crate::services::llm::{LlmClient, LlmLogger};
use crate::services::pdf::{PdfExtractor, TextChunker};
use crate::services::terms::{TermExtractor, OccurrenceTracker, DefinitionGenerator};
use crate::services::translation::TranslationService;
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::path::Path;
use tracing::{debug, error, info, warn};

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

        // Step 2: Extract text from PDF (FR-005, FR-007)
        let file_path = Path::new(&paper.file_path);
        let extraction_result = match PdfExtractor::extract_with_recovery(file_path) {
            Ok(result) => {
                // Log any warnings from extraction
                for warning in &result.warnings {
                    warn!("PDF extraction warning for {}: {}", paper_id, warning);
                }

                // Check if extraction is usable
                if !PdfExtractor::is_extraction_usable(&result) {
                    warn!("PDF extraction produced insufficient text for {}", paper_id);
                    if result.text.is_empty() {
                        Paper::update_status(&self.pool, paper_id, PaperStatus::Failed).await?;
                        anyhow::bail!("No text could be extracted from PDF. The file may be corrupted, encrypted, or contain only scanned images.");
                    }
                }

                // Log if partial extraction
                if result.is_partial {
                    warn!("PDF extraction is partial for {} - some content may be missing", paper_id);
                }

                result
            }
            Err(e) => {
                error!("PDF extraction completely failed for {}: {}", paper_id, e);
                Paper::update_status(&self.pool, paper_id, PaperStatus::Failed).await?;
                return Err(e);
            }
        };

        if !extraction_result.failed_pages.is_empty() {
            warn!(
                "PDF extraction had {} failed pages for paper {}",
                extraction_result.failed_pages.len(),
                paper_id
            );
        }

        // Step 3: Chunk text (even if partial)
        let chunker = TextChunker::default();
        let chunks = if extraction_result.text.is_empty() {
            warn!("No text to chunk for paper {}", paper_id);
            vec![]
        } else {
            chunker.chunk(&extraction_result.text)
        };

        if chunks.is_empty() {
            warn!("No chunks created for paper {} - cannot proceed with translation", paper_id);
            Paper::update_status(&self.pool, paper_id, PaperStatus::Failed).await?;
            anyhow::bail!("No processable content found in PDF");
        }

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

        // Step 5: Translate chunks in parallel (FR-014)
        let llm_logger = LlmLogger::new(paper_id)?;

        // Collect chunk texts for parallel processing
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();

        info!("Translating {} chunks in parallel for paper {}", chunk_texts.len(), paper_id);

        // Translate all chunks in parallel (up to 10 concurrent per FR-014)
        let results = self.translation_service.translate_chunks(chunk_texts).await;

        // Process translation results
        let db_chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;

        for (index, result) in results.into_iter().enumerate() {
            let chunk = &chunks[index];
            let db_chunk = db_chunks.get(index);

            if let Some(db_chunk) = db_chunk {
                match result {
                    Ok(translation_result) => {
                        Chunk::update_translation(
                            &self.pool,
                            &db_chunk.id,
                            translation_result.translated_text.clone(),
                        )
                        .await?;

                        // Log successful translation
                        llm_logger
                            .log_translation(
                                &db_chunk.id,
                                &chunk.text,
                                Some(&translation_result.translated_text),
                                None,
                                translation_result.duration.as_millis(),
                            )
                            .ok();

                        debug!("Translated chunk {} for paper {}", chunk.index, paper_id);
                    }
                    Err(e) => {
                        warn!("Translation failed for chunk {}: {}", chunk.index, e);

                        // Update chunk status to failed
                        Chunk::update_status(&self.pool, &db_chunk.id, "failed", Some(e.to_string())).await.ok();

                        // Log failed translation
                        llm_logger
                            .log_translation(
                                &db_chunk.id,
                                &chunk.text,
                                None,
                                Some(e.to_string()),
                                0,
                            )
                            .ok();

                        // FR-033: Preserve partial results - continue processing other chunks
                    }
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

    /// Extract terms from paper source text in parallel (FR-014)
    async fn extract_terms(&self, paper_id: &str) -> Result<()> {
        use futures::stream::{self, StreamExt};

        let term_extractor = TermExtractor::new(self.llm_client.clone(), self.pool.clone());

        let chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;

        // Filter chunks with translations
        let chunks_to_process: Vec<_> = chunks
            .into_iter()
            .filter(|chunk| chunk.trans_html.is_some())
            .collect();

        info!("Extracting terms from {} chunks in parallel", chunks_to_process.len());

        // Extract terms from each chunk in parallel (up to 10 concurrent per FR-014)
        let results: Vec<_> = stream::iter(chunks_to_process)
            .map(|chunk| {
                let extractor = term_extractor.clone();
                let paper_id = paper_id.to_string();
                async move {
                    let chunk_id = chunk.id.clone();
                    match extractor.extract_terms(&chunk.src_text).await {
                        Ok(extracted_terms) => {
                            extractor
                                .store_terms(&paper_id, &chunk_id, extracted_terms)
                                .await
                                .map(|_| chunk_id.clone())
                        }
                        Err(e) => Err(e),
                    }
                }
            })
            .buffer_unordered(10) // Max 10 concurrent (FR-014)
            .collect()
            .await;

        // Log results
        let mut success_count = 0;
        let mut error_count = 0;
        for result in results {
            match result {
                Ok(chunk_id) => {
                    debug!("Extracted terms from chunk {}", chunk_id);
                    success_count += 1;
                }
                Err(e) => {
                    warn!("Failed to extract terms: {}", e);
                    error_count += 1;
                }
            }
        }

        info!(
            "Term extraction completed: {} succeeded, {} failed",
            success_count, error_count
        );

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

    /// Generate definitions for terms without definitions in parallel (FR-014)
    async fn generate_definitions(&self, paper_id: &str) -> Result<()> {
        use futures::stream::{self, StreamExt};

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

        // Filter terms that don't have definitions yet
        let mut terms_to_generate = Vec::new();
        for term_id in term_ids {
            match crate::models::Definition::find_by_term_id(&self.pool, &term_id).await {
                Ok(Some(_)) => {
                    debug!("Definition already exists for term {}", term_id);
                }
                Ok(None) => {
                    // Need to generate definition
                    if let Ok(term) = Term::find_by_id(&self.pool, &term_id).await {
                        terms_to_generate.push(term);
                    }
                }
                Err(e) => {
                    warn!("Failed to check definition for term {}: {}", term_id, e);
                }
            }
        }

        info!("Generating definitions for {} terms in parallel", terms_to_generate.len());

        // Generate definitions in parallel (up to 10 concurrent per FR-014)
        let results: Vec<_> = stream::iter(terms_to_generate)
            .map(|term| {
                let generator = def_generator.clone();
                async move {
                    let term_name = term.lemma_en.clone();
                    match generator
                        .generate_and_store(&term.id, &term.lemma_en, &term.lemma_ja, None)
                        .await
                    {
                        Ok(_) => Ok(term_name),
                        Err(e) => Err((term_name, e)),
                    }
                }
            })
            .buffer_unordered(10) // Max 10 concurrent (FR-014)
            .collect()
            .await;

        // Log results
        let mut success_count = 0;
        let mut error_count = 0;
        for result in results {
            match result {
                Ok(term_name) => {
                    info!("Generated definition for term {}", term_name);
                    success_count += 1;
                }
                Err((term_name, e)) => {
                    warn!("Failed to generate definition for {}: {}", term_name, e);
                    error_count += 1;
                }
            }
        }

        info!(
            "Definition generation completed: {} succeeded, {} failed",
            success_count, error_count
        );

        Ok(())
    }
}
