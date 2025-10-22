use crate::models::Definition;
use crate::services::llm::LlmClient;
use anyhow::{Result, Context};
use sqlx::SqlitePool;

/// Definition generation service using LLM (spec.md:FR-021)
#[derive(Clone)]
pub struct DefinitionGenerator {
    llm_client: LlmClient,
    pool: SqlitePool,
}

impl DefinitionGenerator {
    pub fn new(llm_client: LlmClient, pool: SqlitePool) -> Self {
        Self { llm_client, pool }
    }

    /// Generate Japanese definition for a term using LLM
    pub async fn generate_definition(
        &self,
        term_en: &str,
        term_ja: &str,
        context: Option<&str>,
    ) -> Result<String> {
        let prompt = if let Some(ctx) = context {
            format!(
                r#"Generate a concise Japanese definition (2-3 sentences) for the technical term:

English: {}
Japanese: {}

Context from paper:
{}

Write the definition in Japanese, targeting ML researchers. Include:
1. What the concept is
2. Key characteristics
3. How it's commonly used

Return ONLY the Japanese definition text, no additional formatting."#,
                term_en, term_ja, ctx
            )
        } else {
            format!(
                r#"Generate a concise Japanese definition (2-3 sentences) for the technical term:

English: {}
Japanese: {}

Write the definition in Japanese, targeting ML researchers. Include:
1. What the concept is
2. Key characteristics
3. How it's commonly used

Return ONLY the Japanese definition text, no additional formatting."#,
                term_en, term_ja
            )
        };

        use crate::services::llm::Message;
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

        let definition = self.llm_client.chat_completion(messages, None, Some(500)).await
            .context("Failed to generate definition from LLM")?;

        Ok(definition.trim().to_string())
    }

    /// Store or update definition for a term
    pub async fn store_definition(
        &self,
        term_id: &str,
        definition_text: String,
        provider: String,
    ) -> Result<()> {
        // Check if definition already exists
        let existing = Definition::find_by_term_id(&self.pool, term_id).await?;

        if existing.is_some() {
            // Update existing definition
            Definition::update(&self.pool, term_id, definition_text, provider)
                .await
                .context("Failed to update definition")?;
        } else {
            // Create new definition
            Definition::create(&self.pool, term_id.to_string(), definition_text, provider)
                .await
                .context("Failed to create definition")?;
        }

        Ok(())
    }

    /// Generate and store definition for a term
    pub async fn generate_and_store(
        &self,
        term_id: &str,
        term_en: &str,
        term_ja: &str,
        context: Option<&str>,
    ) -> Result<String> {
        let definition_text = self.generate_definition(term_en, term_ja, context).await?;

        let provider = format!("llm:{}", self.llm_client.model_name());

        self.store_definition(term_id, definition_text.clone(), provider).await?;

        Ok(definition_text)
    }
}
