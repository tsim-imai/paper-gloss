use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error};

/// LLM request/response logger (FR-016: Constitution Principle V)
pub struct LlmLogger {
    base_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmLogEntry {
    timestamp: String,
    operation: String,
    request: serde_json::Value,
    response: Option<serde_json::Value>,
    error: Option<String>,
    duration_ms: Option<u128>,
}

impl LlmLogger {
    /// Create new LLM logger for a specific paper
    pub fn new(paper_id: &str) -> Result<Self> {
        let base_dir = PathBuf::from(format!("artifacts/papers/{}/llm_logs", paper_id));
        fs::create_dir_all(&base_dir)
            .context("Failed to create LLM logs directory")?;

        Ok(Self { base_dir })
    }

    /// Log an LLM request/response
    pub fn log(
        &self,
        operation: &str,
        request: serde_json::Value,
        response: Option<serde_json::Value>,
        error: Option<String>,
        duration_ms: Option<u128>,
    ) -> Result<()> {
        let entry = LlmLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            operation: operation.to_string(),
            request,
            response,
            error,
            duration_ms,
        };

        let log_file = self.base_dir.join(format!("{}.jsonl", operation));
        let log_line = serde_json::to_string(&entry)?;

        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .and_then(|mut file| {
                use std::io::Write;
                writeln!(file, "{}", log_line)
            })
            .context("Failed to write LLM log entry")?;

        debug!("Logged LLM {} to {:?}", operation, log_file);

        Ok(())
    }

    /// Log a translation request/response
    pub fn log_translation(
        &self,
        chunk_id: &str,
        source_text: &str,
        translated_text: Option<&str>,
        error: Option<String>,
        duration_ms: u128,
    ) -> Result<()> {
        let request = serde_json::json!({
            "chunk_id": chunk_id,
            "source_text": source_text,
        });

        let response = translated_text.map(|text| serde_json::json!({
            "translated_text": text,
        }));

        self.log("translation", request, response, error, Some(duration_ms))
    }

    /// Log a term extraction request/response
    pub fn log_term_extraction(
        &self,
        text: &str,
        terms: Option<Vec<String>>,
        error: Option<String>,
        duration_ms: u128,
    ) -> Result<()> {
        let request = serde_json::json!({
            "text": text,
        });

        let response = terms.map(|t| serde_json::json!({
            "terms": t,
        }));

        self.log("term_extraction", request, response, error, Some(duration_ms))
    }

    /// Log a definition generation request/response
    pub fn log_definition(
        &self,
        term: &str,
        definition: Option<&str>,
        error: Option<String>,
        duration_ms: u128,
    ) -> Result<()> {
        let request = serde_json::json!({
            "term": term,
        });

        let response = definition.map(|def| serde_json::json!({
            "definition": def,
        }));

        self.log("definition", request, response, error, Some(duration_ms))
    }
}

impl Drop for LlmLogger {
    fn drop(&mut self) {
        debug!("LlmLogger for {:?} dropped", self.base_dir);
    }
}
