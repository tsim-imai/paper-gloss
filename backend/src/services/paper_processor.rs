use crate::models::{Chunk, Paper, PaperStatus};
use crate::services::llm::LlmLogger;
use crate::services::pdf::{PdfExtractor, TextChunker};
use crate::services::translation::TranslationService;
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::path::Path;
use tracing::{debug, info, warn};

/// Paper processing orchestrator (FR-031, FR-032, FR-033)
/// Coordinates: extraction → chunking → translation
pub struct PaperProcessor {
    pool: SqlitePool,
    translation_service: TranslationService,
}

impl PaperProcessor {
    pub fn new(pool: SqlitePool) -> Result<Self> {
        Ok(Self {
            pool,
            translation_service: TranslationService::new()?,
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
        } else if translated_count < total_count {
            warn!(
                "Partial translation: {}/{} chunks for paper {}",
                translated_count, total_count, paper_id
            );
            Paper::update_status(&self.pool, paper_id, PaperStatus::Completed).await?;
        } else {
            info!("Successfully processed paper {}", paper_id);
            Paper::update_status(&self.pool, paper_id, PaperStatus::Completed).await?;
        }

        Ok(())
    }

    /// Retry translation for a specific chunk (FR-032)
    pub async fn retry_chunk(&self, chunk_id: &str) -> Result<()> {
        info!("Retrying chunk translation: {}", chunk_id);

        let chunk = Chunk::find_by_id(&self.pool, chunk_id)
            .await
            .context("Failed to find chunk")?;

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

                llm_logger
                    .log_translation(chunk_id, &chunk.src_text, None, Some(e.to_string()), 0)
                    .ok();

                Err(e)
            }
        }
    }
}
