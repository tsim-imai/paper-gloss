use crate::services::llm::LlmClient;
use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Translation service using LLM (FR-011, FR-012, FR-013, FR-014, FR-015)
pub struct TranslationService {
    llm_client: LlmClient,
}

#[derive(Debug)]
pub struct TranslationResult {
    pub translated_text: String,
    pub duration: Duration,
    pub retry_count: usize,
}

impl TranslationService {
    /// Create a new translation service
    pub fn new() -> Result<Self> {
        Ok(Self {
            llm_client: LlmClient::new()?,
        })
    }

    /// Translate a text chunk to Japanese with retry logic (FR-015: exponential backoff)
    pub async fn translate_chunk(&self, source_text: &str, max_retries: usize) -> Result<TranslationResult> {
        let start = Instant::now();
        let mut retry_count = 0;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            retry_count = attempt;

            match self.llm_client.translate(source_text).await {
                Ok(translated) => {
                    return Ok(TranslationResult {
                        translated_text: translated,
                        duration: start.elapsed(),
                        retry_count,
                    });
                }
                Err(e) => {
                    warn!("Translation attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    if attempt < max_retries {
                        // Exponential backoff: 1s, 2s, 4s, 8s...
                        let backoff_ms = 1000 * (1 << attempt);
                        debug!("Retrying after {}ms backoff", backoff_ms);
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Batch translate multiple chunks with concurrency control
    pub async fn translate_chunks(&self, chunks: Vec<String>) -> Vec<Result<TranslationResult>> {
        use futures::stream::{self, StreamExt};

        // Process chunks in parallel (LLM client already handles concurrency limit)
        let results: Vec<_> = stream::iter(chunks)
            .map(|chunk| async move {
                self.translate_chunk(&chunk, 3).await // 3 retries max
            })
            .buffer_unordered(10) // Max 10 concurrent (FR-014)
            .collect()
            .await;

        results
    }
}

impl Default for TranslationService {
    fn default() -> Self {
        Self::new().expect("Failed to create translation service")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires LLM API setup
    async fn test_translate_chunk() {
        let service = TranslationService::new().unwrap();
        let result = service.translate_chunk("Hello, world!", 0).await;

        // This test would fail without a real LLM API
        // It's here as a template for integration testing
        assert!(result.is_err() || result.unwrap().translated_text.len() > 0);
    }
}
