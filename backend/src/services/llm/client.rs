use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{debug, error};

/// OpenAI-compatible LLM client with concurrency limiting
#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    api_base: String,
    api_key: String,
    semaphore: std::sync::Arc<Semaphore>,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

impl LlmClient {
    /// Create new LLM client with concurrency limit (max 10 parallel requests per FR-014)
    /// Defaults to localhost:8000 if environment variables are not set
    pub fn new() -> Result<Self> {
        let api_base = env::var("AI_API_BASE")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());
        let api_key = env::var("AI_API_KEY")
            .unwrap_or_else(|_| "test-api-key".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            api_base,
            api_key,
            semaphore: std::sync::Arc::new(Semaphore::new(10)), // FR-014: Max 10 concurrent requests
        })
    }

    /// Get the model name being used
    pub fn model_name(&self) -> &str {
        "gpt-4"
    }

    /// Send chat completion request to LLM API
    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model: Option<String>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        // Acquire semaphore permit (limits concurrency)
        let _permit = self.semaphore.acquire().await?;

        let model = model.unwrap_or_else(|| "gpt-4".to_string());
        let url = format!("{}/chat/completions", self.api_base);

        let request_body = ChatRequest {
            model,
            messages,
            temperature: 0.7,
            max_tokens,
        };

        debug!("LLM request: {:?}", request_body);

        // FR-015: Exponential backoff retry on transient failures
        let max_retries = 5usize;
        let mut attempt = 0usize;
        loop {
            attempt += 1;
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .context("Failed to send LLM request")?;

            let status = response.status();

            if status == StatusCode::OK {
                let body = response.text().await?;
                let chat_response: ChatResponse = serde_json::from_str(&body)
                    .context("Failed to parse LLM response")?;
                let content = chat_response
                    .choices
                    .first()
                    .and_then(|choice| Some(choice.message.content.clone()))
                    .context("LLM response missing content")?;
                debug!("LLM response: {}", content);
                return Ok(content);
            }

            // Decide if retryable
            let retryable = status == StatusCode::TOO_MANY_REQUESTS
                || status.is_server_error();

            // Compute backoff
            if retryable && attempt < max_retries {
                // Honor Retry-After for 429 if present
                let mut delay = None;
                if status == StatusCode::TOO_MANY_REQUESTS {
                    if let Some(h) = response.headers().get("retry-after") {
                        if let Ok(v) = h.to_str() {
                            if let Ok(secs) = v.parse::<u64>() {
                                delay = Some(Duration::from_secs(secs));
                            } else if let Ok(dt) = httpdate::parse_http_date(v) {
                                let now = std::time::SystemTime::now();
                                if let Ok(dur) = dt.duration_since(now) {
                                    delay = Some(dur);
                                }
                            }
                        }
                    }
                }

                // Default exponential backoff with jitter
                let base = Duration::from_millis(100);
                let exp = base * (1u32 << (attempt - 1));
                let backoff = delay.unwrap_or(exp);
                let jitter_ms = (backoff.as_millis() as i64) / 5; // ±20%
                let jitter = if jitter_ms > 0 {
                    let r = (rand::random::<i32>() as i64).abs() % (2 * jitter_ms + 1) - jitter_ms;
                    if r >= 0 { Duration::from_millis((backoff.as_millis() as i64 + r) as u64) } else { Duration::from_millis((backoff.as_millis() as i64 - r.abs()) as u64) }
                } else {
                    backoff
                };

                debug!("LLM retry attempt {} after {:?} (status={})", attempt, jitter, status);
                sleep(jitter).await;
                continue;
            } else {
                let body = response.text().await.unwrap_or_default();
                error!("LLM API error ({}): {}", status, body);
                anyhow::bail!("LLM API returned error status {}: {}", status, body);
            }
        }
    }

    /// Translate text chunk to Japanese
    pub async fn translate(&self, source_text: &str) -> Result<String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a professional English-to-Japanese translator specializing in machine learning papers. Translate the following text to Japanese, preserving mathematical notation, symbols, and reference labels. Maintain the original structure and formatting.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: source_text.to_string(),
            },
        ];

        self.chat_completion(messages, None, Some(4000)).await
    }

    /// Extract machine learning terms from text
    pub async fn extract_terms(&self, text: &str) -> Result<Vec<String>> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are an expert in machine learning terminology. Extract all machine learning-related technical terms from the provided text. Return only the terms as a JSON array of strings, with no additional explanation.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: text.to_string(),
            },
        ];

        let response = self.chat_completion(messages, None, Some(2000)).await?;
        let terms: Vec<String> = serde_json::from_str(&response)
            .context("Failed to parse term extraction response as JSON array")?;

        Ok(terms)
    }

    /// Generate Japanese definition for a term (2-3 sentences)
    pub async fn generate_definition(&self, term_en: &str, context: Option<&str>) -> Result<String> {
        let prompt = if let Some(ctx) = context {
            format!(
                "Define the machine learning term \"{}\" in Japanese. Context: {}. Provide a 2-3 sentence explanation in Japanese.",
                term_en, ctx
            )
        } else {
            format!(
                "Define the machine learning term \"{}\" in Japanese. Provide a 2-3 sentence explanation in Japanese.",
                term_en
            )
        };

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are an expert in machine learning. Provide concise, accurate Japanese definitions for ML terms.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        self.chat_completion(messages, None, Some(500)).await
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new().expect("Failed to create LLM client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use serial_test::serial;
    use serde_json::json;

    #[tokio::test]
    #[serial]
    async fn openai_chat_completions_happy_path() {
        let server = MockServer::start_async().await;

        server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/v1/chat/completions")
                    .header("authorization", "Bearer sk-test");
                then.status(200).json_body(json!({
                    "id": "chatcmpl-test",
                    "object": "chat.completion",
                    "created": 0,
                    "model": "gpt-4",
                    "choices": [{
                        "index": 0,
                        "message": {"role": "assistant", "content": "こんにちは"},
                        "finish_reason": "stop"
                    }],
                    "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
                }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");

        let client = LlmClient::new().unwrap();
        let messages = vec![
            Message { role: "system".into(), content: "sys".into() },
            Message { role: "user".into(), content: "hello".into() },
        ];
        let out = client.chat_completion(messages, Some("gpt-4".into()), Some(32)).await.unwrap();
        assert_eq!(out, "こんにちは");
    }

    #[tokio::test]
    #[serial]
    async fn openai_chat_completions_error_status_surfaces() {
        let server = MockServer::start_async().await;

        server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(500).json_body(json!({
                    "error": {"message": "server error", "type": "server_error"}
                }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");

        let client = LlmClient::new().unwrap();
        let messages = vec![Message { role: "user".into(), content: "hello".into() }];
        let err = client.chat_completion(messages, None, Some(16)).await.err();
        assert!(err.is_some());
    }

    #[tokio::test]
    #[serial]
    async fn translate_and_extract_terms_use_openai_contract() {
        let server = MockServer::start_async().await;

        // translate
        server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/v1/chat/completions")
                    .body_contains("professional English-to-Japanese translator");
                then.status(200).json_body(json!({
                    "id": "chatcmpl-1",
                    "object": "chat.completion",
                    "created": 0,
                    "model": "gpt-4",
                    "choices": [{
                        "index": 0,
                        "message": {"role": "assistant", "content": "和訳テキスト"},
                        "finish_reason": "stop"
                    }]
                }));
            })
            .await;

        // extract_terms
        server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/v1/chat/completions")
                    .body_contains("Extract all machine learning-related technical terms");
                let terms_json = "[\"ニューラルネットワーク\",\"蒸留\"]";
                then.status(200).json_body(json!({
                    "id": "chatcmpl-2",
                    "object": "chat.completion",
                    "created": 0,
                    "model": "gpt-4",
                    "choices": [{
                        "index": 0,
                        "message": {"role": "assistant", "content": terms_json},
                        "finish_reason": "stop"
                    }]
                }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");

        let client = LlmClient::new().unwrap();
        let jp = client.translate("Some text").await.unwrap();
        assert_eq!(jp, "和訳テキスト");

        let terms = client.extract_terms("foo bar").await.unwrap();
        assert_eq!(terms, vec!["ニューラルネットワーク", "蒸留"]);
    }

    #[tokio::test]
    #[serial]
    async fn fr014_concurrency_is_limited_to_10() {
        use std::time::Instant;

        let server = MockServer::start_async().await;
        server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(200)
                    .delay(std::time::Duration::from_millis(50))
                    .json_body(json!({
                        "id": "chatcmpl",
                        "object": "chat.completion",
                        "created": 0,
                        "model": "gpt-4",
                        "choices": [{
                            "index": 0,
                            "message": {"role": "assistant", "content": "ok"},
                            "finish_reason": "stop"
                        }]
                    }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");
        let client = LlmClient::new().unwrap();

        // Fire 20 concurrent requests; with limit 10 and 50ms per request,
        // elapsed should be roughly >= 100ms (two waves).
        use futures::future::join_all;
        let start = Instant::now();
        let futs = (0..20).map(|_| {
            let c = client.clone();
            async move {
                let msgs = vec![Message { role: "user".into(), content: "ping".into() }];
                let _ = c.chat_completion(msgs, None, Some(8)).await.ok();
            }
        });
        join_all(futs).await;
        let elapsed = start.elapsed();
        assert!(elapsed >= std::time::Duration::from_millis(90), "elapsed {elapsed:?} too short; concurrency limit may be broken");
    }

    #[tokio::test]
    #[serial]
    async fn fr015_retries_500_at_least_three_times() {
        use httpmock::Mock;
        let server = MockServer::start_async().await;

        // Always return 500 for chat completions
        let m: Mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(500).json_body(json!({
                    "error": {"message": "server error", "type": "server_error"}
                }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");
        let client = LlmClient::new().unwrap();

        let _ = client
            .chat_completion(vec![Message { role: "user".into(), content: "hello".into() }], None, Some(8))
            .await
            .err();

        // SPEC FR-015: Expect exponential backoff retries (>=3 attempts)
        assert!(m.hits() >= 3, "expected at least 3 retry attempts, got {}", m.hits());
    }

    #[tokio::test]
    #[serial]
    async fn fr015_retries_429_at_least_three_times() {
        use httpmock::Mock;
        let server = MockServer::start_async().await;

        let m: Mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(429).json_body(json!({
                    "error": {"message": "rate limit", "type": "rate_limit"}
                }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");
        let client = LlmClient::new().unwrap();

        let _ = client
            .chat_completion(vec![Message { role: "user".into(), content: "hello".into() }], None, Some(8))
            .await
            .err();

        assert!(m.hits() >= 3, "expected at least 3 retry attempts, got {}", m.hits());
    }

    #[tokio::test]
    #[serial]
    async fn fr015_honors_retry_after_header_seconds() {
        use httpmock::Mock;
        use std::time::Instant;
        let server = MockServer::start_async().await;

        // Return 429 with Retry-After: 1 (1 second)
        let m: Mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(429)
                    .header("Retry-After", "1")
                    .json_body(json!({
                        "error": {"message": "rate limit", "type": "rate_limit"}
                    }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");
        let client = LlmClient::new().unwrap();

        let start = Instant::now();
        let _ = client
            .chat_completion(vec![Message { role: "user".into(), content: "hello".into() }], None, Some(8))
            .await
            .err();
        let elapsed = start.elapsed();

        // Should wait at least 1 second for the first retry
        assert!(m.hits() >= 2, "expected at least 2 attempts, got {}", m.hits());
        assert!(elapsed >= Duration::from_secs(1), "expected to wait at least 1 second, but only waited {:?}", elapsed);
    }

    #[tokio::test]
    #[serial]
    async fn fr015_honors_retry_after_header_http_date() {
        use httpmock::Mock;
        use std::time::{Instant, SystemTime};
        let server = MockServer::start_async().await;

        // Return 429 with Retry-After as HTTP-date (1 second in future)
        let future_time = SystemTime::now() + Duration::from_secs(1);
        let http_date = httpdate::fmt_http_date(future_time);

        let m: Mock = server
            .mock_async(move |when, then| {
                when.method(POST).path("/v1/chat/completions");
                then.status(429)
                    .header("Retry-After", http_date.clone())
                    .json_body(json!({
                        "error": {"message": "rate limit", "type": "rate_limit"}
                    }));
            })
            .await;

        std::env::set_var("AI_API_BASE", format!("{}/v1", server.base_url()));
        std::env::set_var("AI_API_KEY", "sk-test");
        let client = LlmClient::new().unwrap();

        let start = Instant::now();
        let _ = client
            .chat_completion(vec![Message { role: "user".into(), content: "hello".into() }], None, Some(8))
            .await
            .err();
        let elapsed = start.elapsed();

        // Should wait approximately 1 second for the first retry
        assert!(m.hits() >= 2, "expected at least 2 attempts, got {}", m.hits());
        assert!(elapsed >= Duration::from_millis(900), "expected to wait at least 900ms, but only waited {:?}", elapsed);
    }
}
