use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::sync::Semaphore;
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
        let body = response.text().await?;

        if status != StatusCode::OK {
            error!("LLM API error ({}): {}", status, body);
            anyhow::bail!("LLM API returned error status {}: {}", status, body);
        }

        let chat_response: ChatResponse = serde_json::from_str(&body)
            .context("Failed to parse LLM response")?;

        let content = chat_response
            .choices
            .first()
            .and_then(|choice| Some(choice.message.content.clone()))
            .context("LLM response missing content")?;

        debug!("LLM response: {}", content);

        Ok(content)
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
}
