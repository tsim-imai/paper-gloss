use crate::models::{Chunk, Paper, PaperStatus, Term};
use crate::services::llm::{LlmClient, LlmLogger};
use crate::services::pdf::{PdfExtractor, TextChunker};
use crate::services::terms::{DefinitionGenerator, JapaneseTermExtractor, OccurrenceScannerJa};
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

    /// Check if another pipeline is running for this paper (exclusive lock via table)
    async fn check_pipeline_lock(&self, paper_id: &str) -> Result<()> {
        let exists: Option<i64> = sqlx::query_scalar(
            r#"SELECT 1 FROM pipeline_locks WHERE paper_id = ? LIMIT 1"#,
        )
        .bind(paper_id)
        .fetch_optional(&self.pool)
        .await?;
        if exists.is_some() {
            anyhow::bail!("Another pipeline is already running for this paper");
        }
        Ok(())
    }

    /// Acquire pipeline lock by inserting a row (paper-scoped, single lock)
    /// If `set_processing` is true, update paper.status to Processing (used only by Pipeline A: translate)
    async fn acquire_pipeline_lock(&self, paper_id: &str, pipeline: &str, set_processing: bool) -> Result<PaperStatus> {
        use chrono::Utc;
        let paper = Paper::find_by_id(&self.pool, paper_id).await?;
        sqlx::query(
            r#"INSERT INTO pipeline_locks (paper_id, pipeline, locked_at) VALUES (?, ?, ?)"#,
        )
        .bind(paper_id)
        .bind(pipeline)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        if set_processing {
            // Only Pipeline A should move paper into processing
            Paper::update_status(&self.pool, paper_id, PaperStatus::Processing).await?;
        }
        Ok(paper.status.clone())
    }

    /// Release pipeline lock by deleting the row (no status mutation here)
    async fn release_pipeline_lock(&self, paper_id: &str, _pipeline: &str, _previous_status: PaperStatus) -> Result<()> {
        sqlx::query(
            r#"DELETE FROM pipeline_locks WHERE paper_id = ?"#,
        )
        .bind(paper_id)
        .execute(&self.pool)
        .await?;
        Ok(())
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

        // Step 5: Translate plain chunks in parallel (FR-014)
        let llm_logger = LlmLogger::new(paper_id)?;
        info!("Translating {} chunks in parallel for paper {}", chunks.len(), paper_id);
        let source_texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
        let results = self.translation_service.translate_chunks(source_texts).await;

        // Process translation results → render HTML and store occurrences
        let db_chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;

        for (index, result) in results.into_iter().enumerate() {
            let chunk = &chunks[index];
            let db_chunk = db_chunks.get(index);
            
            if let Some(db_chunk) = db_chunk {
                match result {
                    Ok(translation_result) => {
                        // Store translated plain text
                        Chunk::update_translation(&self.pool, &db_chunk.id, translation_result.translated_text.clone()).await?;

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
                        debug!("LLM retries used for chunk {}: {}", chunk.index, translation_result.retry_count);
                        // Recalculate and update paper status immediately
                        let _ = self.recalc_paper_status(paper_id).await;
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

                        // Recalculate and update paper status (may become failed if all failed)
                        let _ = self.recalc_paper_status(paper_id).await;
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

        // Step 6: Extract JP terms (with lemma_en) and register
        info!("Extracting JP terms for paper {}", paper_id);
        if let Err(e) = self.extract_jp_terms_and_register(paper_id).await {
            warn!("JP term extraction failed for paper {}: {}", paper_id, e);
        }

        // Step 7: Scan JP occurrences and store (non-blocking)
        info!("Scanning JP occurrences for paper {}", paper_id);
        if let Err(e) = self.scan_jp_occurrences(paper_id).await {
            warn!("JP occurrence scanning failed for paper {}: {}", paper_id, e);
        }

        // Step 8: Generate definitions for extracted terms (optional)
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

    // Old English-based extraction and tracking functions removed in JP-first flow

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
                // Recalculate paper status after successful retry
                let _ = self.recalc_paper_status(&chunk.paper_id).await;
                Ok(())
            }
            Err(e) => {
                warn!("Chunk retry failed for {}: {}", chunk_id, e);

                // Update status to failed with error message
                Chunk::update_status(&self.pool, chunk_id, "failed", Some(e.to_string())).await?;

                llm_logger
                    .log_translation(chunk_id, &chunk.src_text, None, Some(e.to_string()), 0)
                    .ok();

                // Recalculate status as well (might become failed if all failed)
                let _ = self.recalc_paper_status(&chunk.paper_id).await;
                Err(e)
            }
        }
    }

    /// Generate definitions for terms without definitions in parallel (FR-014)
    /// Returns (success_count, error_count)
    async fn generate_definitions(&self, paper_id: &str) -> Result<(usize, usize)> {
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

        // Generate definitions in parallel (respect AI_MAX_CONCURRENCY)
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
            .buffer_unordered(std::env::var("AI_MAX_CONCURRENCY").ok().and_then(|v| v.parse().ok()).unwrap_or(5))
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

        Ok((success_count, error_count))
    }

    /// Pipeline A: Translate all chunks for a paper
    pub async fn translate_paper(&self, paper_id: &str) -> Result<()> {
        info!("Pipeline A: Starting translation for paper {}", paper_id);

        // Check lock
        self.check_pipeline_lock(paper_id).await?;
        let prev_status = self.acquire_pipeline_lock(paper_id, "translate", true).await?;

        let result = self.translate_paper_internal(paper_id).await;

        // Release lock regardless of result
        let _ = self.release_pipeline_lock(paper_id, "translate", prev_status).await;

        match &result {
            Ok(_) => {
                // Update timestamp
                let _ = Paper::update_translation_run_at(&self.pool, paper_id).await;
                // Recalculate status based on translation results
                let _ = self.recalc_paper_status(paper_id).await;
            }
            Err(_) => {
                // All failed or fatal error → mark failed per spec
                let _ = Paper::update_status(&self.pool, paper_id, PaperStatus::Failed).await;
            }
        }

        result
    }

    async fn translate_paper_internal(&self, paper_id: &str) -> Result<()> {
        use futures::stream::{self, StreamExt};

        let mut chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;

        // If no chunks exist, extract from PDF and create them
        if chunks.is_empty() {
            info!("No chunks found for paper {}. Extracting from PDF...", paper_id);

            let paper = Paper::find_by_id(&self.pool, paper_id).await?;
            let file_path = Path::new(&paper.file_path);

            // Extract text from PDF
            let extraction_result = PdfExtractor::extract_with_recovery(file_path)?;

            // Log warnings
            for warning in &extraction_result.warnings {
                warn!("PDF extraction warning for {}: {}", paper_id, warning);
            }

            // Check if extraction is usable
            if !PdfExtractor::is_extraction_usable(&extraction_result) {
                anyhow::bail!("PDF extraction produced insufficient text. The file may be corrupted, encrypted, or contain only scanned images.");
            }

            if extraction_result.is_partial {
                warn!("PDF extraction is partial for {} - some content may be missing", paper_id);
            }

            // Chunk text
            let chunker = TextChunker::default();
            let text_chunks = chunker.chunk(&extraction_result.text);

            if text_chunks.is_empty() {
                anyhow::bail!("No processable content found in PDF");
            }

            info!("Created {} chunks from PDF for paper {}", text_chunks.len(), paper_id);

            // Save chunks to database
            for chunk in &text_chunks {
                Chunk::create(
                    &self.pool,
                    paper_id.to_string(),
                    chunk.index as i32,
                    chunk.text.clone(),
                    chunk.content_hash.clone(),
                    chunk.token_count.map(|t| t as i32),
                )
                .await?;
            }

            // Reload chunks from database
            chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;
        }

        let llm_logger = LlmLogger::new(paper_id)?;
        info!("Translating {} chunks in parallel for paper {}", chunks.len(), paper_id);

        let conc: usize = std::env::var("AI_MAX_CONCURRENCY").ok().and_then(|v| v.parse().ok()).unwrap_or(5);

        let stream = stream::iter(chunks.clone())
            .map(|chunk| {
                let svc = self.translation_service.clone();
                async move {
                    let res = svc.translate_chunk(&chunk.src_text, 3).await;
                    (chunk, res)
                }
            })
            .buffer_unordered(conc);

        tokio::pin!(stream);
        while let Some((chunk, result)) = stream.next().await {
            match result {
                Ok(translation_result) => {
                    Chunk::update_translation(&self.pool, &chunk.id, translation_result.translated_text.clone()).await?;

                    llm_logger
                        .log_translation(
                            &chunk.id,
                            &chunk.src_text,
                            Some(&translation_result.translated_text),
                            None,
                            translation_result.duration.as_millis(),
                        )
                        .ok();

                    debug!("Translated chunk {} for paper {}", chunk.index, paper_id);
                    debug!("LLM retries used for chunk {}: {}", chunk.index, translation_result.retry_count);
                }
                Err(e) => {
                    warn!("Translation failed for chunk {}: {}", chunk.index, e);

                    Chunk::update_status(&self.pool, &chunk.id, "failed", Some(e.to_string())).await.ok();

                    llm_logger
                        .log_translation(
                            &chunk.id,
                            &chunk.src_text,
                            None,
                            Some(e.to_string()),
                            0,
                        )
                        .ok();
                }
            }

            // Update status incrementally so UI can reflect partial progress
            let _ = self.recalc_paper_status(paper_id).await;
        }

        let translated_count = Chunk::count_translated_by_paper_id(&self.pool, paper_id).await?;

        if translated_count == 0 {
            anyhow::bail!("No chunks were successfully translated");
        }

        let total_count = Chunk::count_by_paper_id(&self.pool, paper_id).await?;
        info!("Pipeline A completed: {}/{} chunks translated", translated_count, total_count);
        Ok(())
    }

    /// Pipeline B: Extract Japanese terms from translated text
    pub async fn extract_terms_jp(&self, paper_id: &str) -> Result<()> {
        info!("Pipeline B: Starting JP term extraction for paper {}", paper_id);

        // Check lock
        self.check_pipeline_lock(paper_id).await?;
        let prev_status = self.acquire_pipeline_lock(paper_id, "extract-terms-jp", false).await?;

        let result = self.extract_jp_terms_and_register(paper_id).await;

        // Update timestamp (even on Ok - 0 terms is valid)
        if result.is_ok() {
            let _ = Paper::update_terms_jp_run_at(&self.pool, paper_id).await;
        }

        // Release lock
        self.release_pipeline_lock(paper_id, "extract-terms-jp", prev_status).await.ok();

        result
    }

    /// Pipeline C: Scan Japanese text for term occurrences
    pub async fn scan_jp(&self, paper_id: &str) -> Result<()> {
        info!("Pipeline C: Starting JP occurrence scanning for paper {}", paper_id);

        // Check lock
        self.check_pipeline_lock(paper_id).await?;
        let prev_status = self.acquire_pipeline_lock(paper_id, "scan-jp", false).await?;

        let result = self.scan_jp_occurrences(paper_id).await;

        // Update timestamp (even on Ok - 0 occurrences is valid)
        if result.is_ok() {
            let _ = Paper::update_scan_jp_run_at(&self.pool, paper_id).await;
        }

        // Release lock
        self.release_pipeline_lock(paper_id, "scan-jp", prev_status).await.ok();

        result
    }

    /// Pipeline D: Generate definitions for extracted terms
    pub async fn generate_definitions_pipeline(&self, paper_id: &str) -> Result<()> {
        info!("Pipeline D: Starting definition generation for paper {}", paper_id);

        // Check lock
        self.check_pipeline_lock(paper_id).await?;
        let prev_status = self.acquire_pipeline_lock(paper_id, "generate-definitions", false).await?;

        let result = self.generate_definitions(paper_id).await;

        // Update timestamp and result_state based on outcome
        match &result {
            Ok((success_count, error_count)) => {
                let result_state = if *success_count > 0 {
                    "completed_nonempty"
                } else if *error_count == 0 {
                    "completed_empty"
                } else {
                    "failed"
                };
                let _ = Paper::update_definitions_run_at(&self.pool, paper_id, result_state).await;
            }
            Err(_) => {
                // On error, mark as failed
                let _ = Paper::update_definitions_run_at(&self.pool, paper_id, "failed").await;
            }
        }

        // Release lock
        self.release_pipeline_lock(paper_id, "generate-definitions", prev_status).await.ok();

        result.map(|_| ())
    }
}

impl PaperProcessor {
    // JP term extraction + registration
    async fn extract_jp_terms_and_register(&self, paper_id: &str) -> Result<()> {
        use crate::models::{Term, TermVariant};
        let extractor = JapaneseTermExtractor::new(self.llm_client.clone());
        let chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;
        let mut buf = String::new();
        for c in &chunks { if let Some(t) = &c.trans_html { buf.push_str(t); buf.push_str("\n\n"); } }
        if buf.trim().is_empty() { return Ok(()); }

        let mut terms = extractor.extract_from_text(&buf, Some(4000)).await.unwrap_or_default();

        // Deduplicate by normalized JA
        use crate::services::terms::normalize_japanese;
        use std::collections::HashSet;
        let mut seen: HashSet<String> = HashSet::new();
        terms.retain(|t| {
            let key = normalize_japanese(&t.lemma_ja);
            if seen.contains(&key) { return false; }
            seen.insert(key);
            true
        });

        fn slugify_en(s: &str) -> String {
            if s.trim().is_empty() { return uuid::Uuid::new_v4().to_string(); }
            let s = s.to_lowercase();
            let mut out = String::with_capacity(s.len());
            for ch in s.chars() { if ch.is_ascii_alphanumeric() { out.push(ch); } else if ch==' '||ch=='_'||ch=='-' { out.push('-'); } }
            while out.contains("--") { out = out.replace("--","-"); }
            out.trim_matches('-').to_string()
        }

        for t in terms.into_iter() {
            let slug = slugify_en(&t.lemma_en);
            let term_id = match Term::find_by_slug(&self.pool, &slug).await {
                Ok(term) => term.id,
                Err(_) => {
                    let term = Term::create(
                        &self.pool,
                        slug.clone(),
                        t.lemma_en.clone(),
                        t.lemma_ja.clone(),
                        t.reading_kana.clone(),
                        t.pos.clone(),
                        None,
                        None,
                    ).await?;
                    term.id
                }
            };
            let _ = TermVariant::create(&self.pool, term_id.clone(), "ja".into(), t.lemma_ja.clone()).await;
            if let Some(vs) = t.variants_ja { for v in vs { let _ = TermVariant::create(&self.pool, term_id.clone(), "ja".into(), v).await; } }
        }
        Ok(())
    }

    // JP scanning and occurrences (non-blocking best-effort)
    async fn scan_jp_occurrences(&self, paper_id: &str) -> Result<()> {
        use crate::models::Occurrence;

        // Clear existing jp-scan occurrences for idempotent re-run with latest dictionary
        Occurrence::delete_by_paper_and_method(&self.pool, paper_id, "jp-scan").await?;
        info!("Cleared existing jp-scan occurrences for paper {}", paper_id);

        let scanner = OccurrenceScannerJa::new(self.pool.clone());
        let chunks = Chunk::find_by_paper_id(&self.pool, paper_id).await?;
        for c in chunks {
            if let Some(ref jp) = c.trans_html { let _ = scanner.scan_chunk(paper_id, &c.id, jp).await; }
        }
        Ok(())
    }

    /// Recalculate paper status based on chunk counts and update immediately.
    async fn recalc_paper_status(&self, paper_id: &str) -> Result<()> {
        let total = crate::models::Chunk::count_by_paper_id(&self.pool, paper_id).await? as i64;
        let translated = crate::models::Chunk::count_translated_by_paper_id(&self.pool, paper_id).await? as i64;
        let failed = crate::models::Chunk::count_failed_by_paper_id(&self.pool, paper_id).await? as i64;

        if total > 0 && translated == total {
            crate::models::Paper::update_status(&self.pool, paper_id, crate::models::PaperStatus::Completed).await?;
        } else if total > 0 && failed == total {
            crate::models::Paper::update_status(&self.pool, paper_id, crate::models::PaperStatus::Failed).await?;
        } else {
            // Keep processing if mixed or pending
            crate::models::Paper::update_status(&self.pool, paper_id, crate::models::PaperStatus::Processing).await?;
        }
        Ok(())
    }
}
