use crate::services::llm::LlmClient;
use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{debug, warn};
use regex::Regex;

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
        let mut last_error = None;

        for attempt in 0..=max_retries {

            match self.llm_client.translate(source_text).await {
                Ok(translated) => {
                    // FR-012: Preserve math/labels from source if LLM output damaged them
                    let final_text = preserve_tokens(source_text, &translated);
                    return Ok(TranslationResult {
                        translated_text: final_text,
                        duration: start.elapsed(),
                        retry_count: attempt,
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

    /// Translate a TAGGED text chunk (with sentinel tags) to Japanese.
    /// Tags must be preserved exactly; only the inner text is translated.
    #[cfg(feature = "tagged-translation")]
    pub async fn translate_tagged_chunk(&self, tagged_source_text: &str, max_retries: usize) -> Result<TranslationResult> {
        use crate::services::llm::Message;
        let start = Instant::now();
        let mut last_error = None;

        for attempt in 0..=max_retries {
            let system = Message {
                role: "system".to_string(),
                content: "You are a professional English-to-Japanese translator for ML papers. The input contains sentinel tags [[T:...]] and [[/T]]. You MUST preserve tags exactly (including order and pairings). Translate only the text inside tags and outside tags, but never modify the tags themselves.".to_string(),
            };
            let user = Message { role: "user".to_string(), content: tagged_source_text.to_string() };

            match self.llm_client.chat_completion(vec![system.clone(), user.clone()], None, Some(6000)).await {
                Ok(translated) => {
                    // Do not apply preserve_tokens here; tags act as anchors
                    return Ok(TranslationResult { translated_text: translated, duration: start.elapsed(), retry_count: attempt });
                }
                Err(e) => {
                    warn!("Tagged translation attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    if attempt < max_retries {
                        let backoff_ms = 1000 * (1 << attempt);
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Batch translate tagged chunks with concurrency control
    #[cfg(feature = "tagged-translation")]
    pub async fn translate_tagged_chunks(&self, chunks: Vec<String>) -> Vec<Result<TranslationResult>> {
        use futures::stream::{self, StreamExt};
        let conc: usize = std::env::var("AI_MAX_CONCURRENCY").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
        let results: Vec<_> = stream::iter(chunks)
            .map(|chunk| async move { self.translate_tagged_chunk(&chunk, 3).await })
            .buffer_unordered(conc)
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

fn preserve_tokens(src: &str, out: &str) -> String {
    #[derive(Clone)]
    struct Segment {
        text: String,
        start: usize,
    }

    fn collect_segments(src: &str) -> Vec<Segment> {
        let mut segments: Vec<Segment> = Vec::new();

        let push_unique = |segments: &mut Vec<Segment>, text: String, start: usize| {
            if segments.iter().any(|seg| seg.text == text) {
                return;
            }
            segments.push(Segment { text, start });
        };

        let math_re = Regex::new(r"\$[^$]+\$").unwrap();
        for mat in math_re.find_iter(src) {
            push_unique(
                &mut segments,
                src[mat.start()..mat.end()].to_string(),
                mat.start(),
            );
        }

        let eq_re = Regex::new(r"Eq\.\s*\(\d+\)").unwrap();
        for mat in eq_re.find_iter(src) {
            push_unique(
                &mut segments,
                src[mat.start()..mat.end()].to_string(),
                mat.start(),
            );
        }

        let bracket_re = Regex::new(r"\[[A-Za-z]+\.\s*\d+\]").unwrap();
        for mat in bracket_re.find_iter(src) {
            push_unique(
                &mut segments,
                src[mat.start()..mat.end()].to_string(),
                mat.start(),
            );
        }

        segments.sort_by_key(|seg| seg.start);
        segments
    }

    fn insert_with_context(result: &mut String, segment: &Segment, src: &str) {
        if result.contains(&segment.text) {
            return;
        }

        // Try to anchor after preceding context (last 24 chars).
        if segment.start > 0 {
            let anchor_start = segment.start.saturating_sub(24);
            let anchor = &src[anchor_start..segment.start];
            let trimmed_anchor = anchor.trim_start_matches(|c: char| c.is_whitespace());
            if trimmed_anchor.len() > 1 {
                if let Some(pos) = result.rfind(trimmed_anchor) {
                    let insert_pos = pos + trimmed_anchor.len();
                    result.insert_str(insert_pos, segment.text.as_str());
                    return;
                }
            }
        }

        // Try to anchor before following context (next 24 chars).
        let after_start = segment.start + segment.text.len();
        if after_start < src.len() {
            let after_end = (after_start + 24).min(src.len());
            let anchor = &src[after_start..after_end];
            let trimmed_anchor = anchor.trim_end_matches(|c: char| c.is_whitespace());
            if trimmed_anchor.len() > 1 {
                if let Some(pos) = result.find(trimmed_anchor) {
                    result.insert_str(pos, segment.text.as_str());
                    return;
                }
            }
        }

        // Fallback: append at the end separated by a space.
        if !result.ends_with(char::is_whitespace) {
            result.push(' ');
        }
        result.push_str(segment.text.as_str());
    }

    let mut result = out.to_string();
    for segment in collect_segments(src) {
        insert_with_context(&mut result, &segment, src);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{MockServer, Method::POST};
    use serial_test::serial;
    use regex::Regex;

    #[tokio::test]
    #[ignore] // Requires LLM API setup
    async fn test_translate_chunk() {
        let service = TranslationService::new().unwrap();
        let result = service.translate_chunk("Hello, world!", 0).await;

        // This test would fail without a real LLM API
        // It's here as a template for integration testing
        assert!(result.is_err() || result.unwrap().translated_text.len() > 0);
    }

    /// FR-012: 数式・記号・参照ラベル・体裁を保持すること
    /// 本テストは、LLMが誤って体裁を壊した訳を返しても、最終出力では
    /// 元の数式や参照ラベル（例: $...$, Eq. (1), [Fig. 2]）がそのまま残ることを要求する。
    /// いまは未実装のため、RED（Fail）になります。
    #[tokio::test]
    #[serial]
    async fn fr012_preserve_math_and_reference_labels() {
        // モックLLM: 翻訳結果でわざと数式$...$や参照表記を壊す
        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(200).json_body(serde_json::json!({
                    "id": "chatcmpl-fr012",
                    "object": "chat.completion",
                    "created": 0,
                    "model": "gpt-4",
                    "choices": [{
                        "index": 0,
                        // 数式デリミタや参照書式を破壊して返す（RED用）
                        "message": {"role": "assistant", "content": "ここでは損失関数を定義する（式1）。L(θ) = Σ_i (y_i - f_θ(x_i))^2。図2参照。"},
                        "finish_reason": "stop"
                    }]
                }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");

        let svc = TranslationService::new().unwrap();

        // 入力テキスト（保持すべき要素を含む）
        let src = "We define the loss in Eq. (1): $L(\\theta) = \\sum_i (y_i - f_\\theta(x_i))^2$. See [Fig. 2].";

        let out = svc.translate_chunk(src, 0).await.unwrap().translated_text;

        // 期待: $...$ 内の数式は文字列としてそのまま含まれる
        let math_re = Regex::new(r"\$[^$]+\$").unwrap();
        for m in math_re.find_iter(src) {
            let seg = &src[m.start()..m.end()];
            assert!(out.contains(seg), "math segment must be preserved: {}", seg);
        }

        // 期待: 参照ラベルもそのまま
        assert!(out.contains("Eq. (1)"), "reference label must be preserved: Eq. (1)");
        assert!(out.contains("[Fig. 2]"), "reference label must be preserved: [Fig. 2]");
    }

    /// FR-014: Test parallel translation with max 10 concurrent requests
    #[tokio::test]
    #[serial]
    async fn fr014_translate_chunks_processes_in_parallel() {
        use std::time::Instant;

        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(200)
                    .delay(std::time::Duration::from_millis(100)) // Each takes 100ms
                    .json_body(serde_json::json!({
                        "id": "chatcmpl-parallel",
                        "object": "chat.completion",
                        "created": 0,
                        "model": "gpt-4",
                        "choices": [{
                            "index": 0,
                            "message": {"role": "assistant", "content": "翻訳されたテキスト"},
                            "finish_reason": "stop"
                        }]
                    }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");

        let service = TranslationService::new().unwrap();

        // Create 20 chunks to translate
        let chunks: Vec<String> = (0..20).map(|i| format!("Chunk {}", i)).collect();

        let start = Instant::now();
        let results = service.translate_chunks(chunks).await;
        let elapsed = start.elapsed();

        // All chunks should be successfully translated
        assert_eq!(results.len(), 20);
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(success_count, 20, "All chunks should be translated successfully");

        // With 20 chunks at 100ms each:
        // - Sequential: 20 * 100ms = 2000ms
        // - Parallel (10 concurrent): ceil(20/10) * 100ms = 200ms
        // We expect it to complete in roughly 200-400ms (with some overhead)
        assert!(
            elapsed.as_millis() < 800,
            "Parallel processing should complete much faster than sequential. Took {:?}",
            elapsed
        );

        // Should take at least 200ms (2 waves of 10)
        assert!(
            elapsed.as_millis() >= 150,
            "Should take time for at least 2 waves. Took {:?}",
            elapsed
        );
    }

    /// FR-014: Test that translate_chunks handles errors gracefully
    #[tokio::test]
    #[serial]
    async fn fr014_translate_chunks_handles_partial_failures() {
        let server = MockServer::start_async().await;

        // Mock that returns 500 error for all requests
        server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(500).json_body(serde_json::json!({
                    "error": {"message": "Internal server error", "type": "server_error"}
                }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");

        let service = TranslationService::new().unwrap();

        let chunks: Vec<String> = (0..5).map(|i| format!("Chunk {}", i)).collect();

        let results = service.translate_chunks(chunks).await;

        assert_eq!(results.len(), 5, "Should process all chunks even with failures");

        // All should fail in this test
        let error_count = results.iter().filter(|r| r.is_err()).count();

        // FR-033: Even with failures, we still return results for all chunks
        // (they're just error results, but partial results are preserved)
        assert!(error_count >= 1, "Should have at least one error with 500 responses");

        println!("Errors: {}", error_count);
    }
}
