use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, error};

/// OpenAI-compatible LLM client with concurrency limiting
pub struct LlmClient {
    client: Client,
    api_base: String,
    api_key: String,
    semaphore: Semaphore,
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
    pub fn new() -> Result<Self> {
        let api_base = env::var("AI_API_BASE")
            .context("AI_API_BASE environment variable not set")?;
        let api_key = env::var("AI_API_KEY")
            .context("AI_API_KEY environment variable not set")?;

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            api_base,
            api_key,
            semaphore: Semaphore::new(10), // FR-014: Max 10 concurrent requests
        })
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
