use crate::services::llm::{LlmClient, Message};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JPExtractedTerm {
    pub lemma_ja: String,
    #[serde(default)]
    pub lemma_en: String,
    #[serde(default)]
    pub reading_kana: Option<String>,
    #[serde(default)]
    pub pos: Option<String>,
    #[serde(default)]
    pub variants_ja: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct JapaneseTermExtractor {
    llm: LlmClient,
}

impl JapaneseTermExtractor {
    pub fn new(llm: LlmClient) -> Self { Self { llm } }

    /// Extract Japanese terms (and lemma_en) from Japanese translated text.
    pub async fn extract_from_text(&self, jp_text: &str, max_tokens: Option<u32>) -> Result<Vec<JPExtractedTerm>> {
        let system = Message {
            role: "system".into(),
            content: "あなたは日本語の科学技術論文から機械学習の専門用語を抽出するアノテータです。出力はJSON配列のみ。各要素は {lemma_ja, lemma_en, reading_kana?, pos?, variants_ja?}。本文の要約や改変はしないでください。スパンや位置は出力しないでください。".into(),
        };
        let user = Message {
            role: "user".into(),
            content: format!(
                "テキスト:\n{}\n\n要件:\n- JSON配列のみを返す\n- 一般語は除外\n- 重複は統合し、代表表記を lemma_ja とする\n- lemma_en も可能な限り付与\n",
                jp_text
            ),
        };

        let resp = self.llm.chat_completion(vec![system, user], None, max_tokens.or(Some(3000))).await?;
        let terms: Vec<JPExtractedTerm> = serde_json::from_str(&resp)
            .context("Failed to parse JP term extraction JSON array")?;
        Ok(terms)
    }
}

