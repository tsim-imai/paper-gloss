use crate::models::Definition;
use crate::services::llm::LlmClient;
use anyhow::{Result, Context};
use sqlx::SqlitePool;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

// ==================== v2 types ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenQuality { pub confidence: Option<f64>, pub flags: Option<Vec<String>> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenMeta { pub provider: Option<String>, pub model: Option<String>, pub prompt_version: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationBlock {
    pub summary: String,
    #[serde(default)]
    pub long: Option<String>,
    #[serde(default)]
    pub usage_examples: Option<Vec<String>>,
    #[serde(default)]
    pub confusables: Option<Vec<String>>,
    #[serde(default)]
    pub see_also: Option<Vec<String>>,
    #[serde(default)]
    pub quality: Option<GenQuality>,
    #[serde(default)]
    pub meta: Option<GenMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionV2Output {
    pub schema_version: String,         // "d2.0"
    pub term_id: String,
    pub lang: String,                   // "ja"
    pub generation: GenerationBlock,
    #[serde(default)]
    pub tags: Option<serde_json::Value>,
    #[serde(default)]
    pub aliases: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub relations: Option<Vec<serde_json::Value>>,
}

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

    /// Generate JSON definition (v2)
    pub async fn generate_definition_v2(
        &self,
        term_en: &str,
        term_ja: &str,
        context: Option<&str>,
    ) -> Result<DefinitionV2Output> {
        use crate::services::llm::Message;
        let system = Message { role: "system".into(), content: "あなたは機械学習分野の日本語テクニカルライターです。出力は必ず指定のJSONスキーマに従います。事実に自信がない場合は不確実フラグを付け、臆測は避けます。定義は初学者にも伝わる簡潔さを優先します。".into() };

        let mut content = format!(
            "対象用語: {} / 英語: {}\n出力: summary(2-3文), usage_examples(1-3), confusables(0-3), see_also(0-5)\n必ず JSON (fields: schema_version='d2.0', lang='ja', generation{{summary,long?,usage_examples?,confusables?,see_also?,quality?}}) で。",
            term_ja, term_en
        );
        if let Some(ctx) = context { content.push_str(&format!("\n用例コンテキスト:\n{}\n", ctx)); }

        let user = Message { role: "user".into(), content };

        let raw = self.llm_client.chat_completion(vec![system, user], None, Some(800)).await?;
        let mut json: DefinitionV2Output = serde_json::from_str(&raw)
            .or_else(|_| serde_json::from_str(&raw.trim_matches('`'))) // tolerate fenced
            .context("Failed to parse v2 definition JSON")?;
        json.lang = "ja".into();
        Ok(json)
    }

    /// Store v2 summary and metadata; append full JSON to artifacts
    pub async fn store_definition_v2(
        &self,
        paper_id: Option<&str>,
        term_id: &str,
        term_en: &str,
        term_ja: &str,
        output: &mut DefinitionV2Output,
    ) -> Result<String> {
        // 1) Write summary into definitions
        let provider = format!("ai:{}", self.llm_client.model_name());
        let summary = output.generation.summary.trim().to_string();
        self.store_definition(term_id, summary.clone(), provider.clone()).await?;

        // 2) Upsert definition_meta
        let confidence = output.generation.quality.as_ref().and_then(|q| q.confidence);
        let flags_json = serde_json::to_string(&output.generation.quality.as_ref().and_then(|q| q.flags.clone()).unwrap_or_default()).ok();
        sqlx::query(
            r#"
            INSERT INTO definition_meta (id, term_id, provider, model, prompt_version, confidence, flags, updated_at)
            VALUES (?, ?, ?, ?, 'd2', ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(term_id) DO UPDATE SET provider=excluded.provider, model=excluded.model, prompt_version='d2', confidence=excluded.confidence, flags=excluded.flags, updated_at=CURRENT_TIMESTAMP
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(term_id)
        .bind(&provider)
        .bind(self.llm_client.model_name())
        .bind(confidence)
        .bind(flags_json)
        .execute(&self.pool)
        .await?;

        // 3) Append JSON to artifacts (paper-scoped or term-scoped)
        let record = serde_json::json!({
            "term_id": term_id,
            "lemma_en": term_en,
            "lemma_ja": term_ja,
            "generated": output,
        });
        let line = serde_json::to_string(&record)?;
        let path: PathBuf = if let Some(pid) = paper_id {
            let dir = PathBuf::from(format!("artifacts/papers/{}", pid));
            fs::create_dir_all(&dir).ok();
            dir.join("definitions.d2.jsonl")
        } else {
            let dir = PathBuf::from(format!("artifacts/terms/{}", term_id));
            fs::create_dir_all(&dir).ok();
            dir.join("definitions.d2.jsonl")
        };
        fs::OpenOptions::new().create(true).append(true).open(&path).and_then(|mut f| { use std::io::Write; writeln!(f, "{}", line) })?;

        Ok(summary)
    }

    /// Generate and store v2 definition in one call
    pub async fn generate_and_store_v2(
        &self,
        paper_id: Option<&str>,
        term_id: &str,
        term_en: &str,
        term_ja: &str,
        context: Option<&str>,
    ) -> Result<String> {
        let mut json = self.generate_definition_v2(term_en, term_ja, context).await?;
        json.schema_version = "d2.0".into();
        json.term_id = term_id.to_string();
        let summary = self.store_definition_v2(paper_id, term_id, term_en, term_ja, &mut json).await?;
        Ok(summary)
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
        self.generate_and_store_v2(None, term_id, term_en, term_ja, context).await
    }
}
