use crate::services::llm::{LlmClient, Message};
use anyhow::{Context, Result};
use regex::Regex;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TagInfo {
    pub id: String,
    pub surface: String,
    #[serde(default)]
    pub lemma_en: Option<String>,
    #[serde(default)]
    pub pos: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TaggingResult {
    pub tagged_text: String,
    pub tags: Vec<TagInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct TagHints {
    pub priority_terms: Vec<String>,
    pub stop_terms: Vec<String>,
    pub max_tags: usize, // default 200
}

#[derive(Clone)]
pub struct TermTagger {
    llm: LlmClient,
}

impl TermTagger {
    pub fn new(llm: LlmClient) -> Self {
        Self { llm }
    }

    /// Ask LLM to insert sentinel tags into the source English text.
    pub async fn annotate(&self, src_text: &str, hints: Option<TagHints>) -> Result<TaggingResult> {
        let TagHints { priority_terms, stop_terms, max_tags } = hints.unwrap_or_else(|| TagHints { max_tags: 200, ..Default::default() });

        let system = Message {
            role: "system".to_string(),
            content: concat!(
                "You are an annotator for English scientific papers.",
                " Insert sentinel tags [[T:UUID]] and [[/T]] around technical terms and named entities.",
                " IMPORTANT RULES: Do not modify any characters outside tags.",
                " Do not reflow text. Do not add or remove spaces.",
                " No nested tags. Each tag must have a unique UUID.",
                " Return JSON with fields: tagged_text, tags:[{id,surface,lemma_en?,pos?}]."
            ).to_string(),
        };

        let user = Message { role: "user".to_string(), content: format!(
            "Text:\n{}\n\nPriority terms (hints, optional): {}\nStop terms (avoid tagging): {}\nMax tags: {}\nConstraints: longest-match, prioritize priority terms, avoid function words, no nested tags.",
            src_text,
            if priority_terms.is_empty() { "(none)".to_string() } else { priority_terms.join(", ") },
            if stop_terms.is_empty() { "(none)".to_string() } else { stop_terms.join(", ") },
            if max_tags == 0 { 200 } else { max_tags },
        )};

        let resp = self.llm.chat_completion(vec![system, user], None, Some(4000)).await?;

        // Parse JSON
        let mut result: TaggingResult = serde_json::from_str(&resp)
            .context("Failed to parse tagger JSON (expected {tagged_text,tags[]})")?;

        // Validate and normalize
        validate_tagged(src_text, &result).context("Invalid tagging output from LLM")?;

        // Ensure tags list contains unique ids and surfaces exist in text
        dedup_and_sanitize_tags(&mut result);

        Ok(result)
    }
}

fn validate_tagged(src: &str, res: &TaggingResult) -> Result<()> {
    // 1) Must match original when tags removed (UTF-8 safe)
    let stripped = strip_sentinels(&res.tagged_text);
    anyhow::ensure!(stripped == src, "tagged_text without tags must equal source");

    // 2) Validate tag structure at character boundaries
    fn is_uuid_like(s: &str) -> bool {
        // simple 8-4-4-4-12 check
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 5 { return false; }
        let lens = [8,4,4,4,12];
        for (p, &l) in parts.iter().zip(lens.iter()) {
            if p.len() != l || !p.chars().all(|c| c.is_ascii_hexdigit()) { return false; }
        }
        true
    }

    let mut stack: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let s = &res.tagged_text;
    let mut i: usize = 0;
    while i < s.len() {
        if s[i..].starts_with("[[T:") {
            // parse id until ']]'
            let after = &s[i+4..];
            if let Some(end) = after.find("]]") {
                let id = &after[..end];
                anyhow::ensure!(is_uuid_like(id), "invalid tag id: {}", id);
                anyhow::ensure!(!seen.contains(id), "duplicate tag id: {}", id);
                seen.insert(id.to_string());
                stack.push(id.to_string());
                i += 4 + end + 2; // '[[T:' + id + ']]'
                continue;
            } else {
                anyhow::bail!("unterminated tag header starting at {}", i);
            }
        }
        if s[i..].starts_with("[[/T]]") {
            anyhow::ensure!(!stack.is_empty(), "closing tag without open");
            stack.pop();
            i += 6;
            continue;
        }
        // advance by one character (not byte)
        let ch_len = s[i..].chars().next().unwrap().len_utf8();
        i += ch_len;
    }
    anyhow::ensure!(stack.is_empty(), "unclosed tags");
    Ok(())
}

fn dedup_and_sanitize_tags(result: &mut TaggingResult) {
    use std::collections::HashSet;
    let mut seen: HashSet<String> = HashSet::new();
    result.tags.retain(|t| {
        if t.id.len() < 8 { return false; }
        if seen.contains(&t.id) { return false; }
        seen.insert(t.id.clone());
        true
    });
}

pub fn strip_sentinels(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i: usize = 0;
    while i < s.len() {
        if s[i..].starts_with("[[T:") {
            if let Some(end) = s[i+4..].find("]]") {
                i += 4 + end + 2; // skip '[[T:' + id + ']]'
                continue;
            }
        }
        if s[i..].starts_with("[[/T]]") { i += 6; continue; }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[derive(Debug, Clone)]
pub struct TagSpan {
    pub id: String,
    pub start: usize,
    pub end: usize,
    pub surface: String,
}

/// Parse tagged text into plain-text spans (offsets relative to text with tags removed)
pub fn parse_tagged_spans(tagged: &str) -> Result<Vec<TagSpan>> {
    let mut spans: Vec<TagSpan> = Vec::new();
    let mut plain_idx: usize = 0; // count in characters
    let mut i: usize = 0; // byte index at char boundary
    let mut stack: Vec<(String, usize, usize)> = Vec::new(); // (id, start_plain_idx, start_i)

    while i < tagged.len() {
        if tagged[i..].starts_with("[[T:") {
            if let Some(close) = tagged[i+4..].find("]]") {
                let id = &tagged[i + 4 .. i + 4 + close];
                stack.push((id.to_string(), plain_idx, i));
                i += 4 + close + 2; // skip header
                continue;
            }
        }
        if tagged[i..].starts_with("[[/T]]") {
            let (id, start_plain, start_i) = stack.pop().ok_or_else(|| anyhow::anyhow!("closing tag without open"))?;
            let surface = strip_sentinels(&tagged[start_i..i]);
            spans.push(TagSpan { id, start: start_plain, end: plain_idx, surface });
            i += 6; // skip closing
            continue;
        }
        let ch = tagged[i..].chars().next().unwrap();
        i += ch.len_utf8();
        plain_idx += 1;
    }

    if !stack.is_empty() { anyhow::bail!("unclosed tags"); }
    Ok(spans)
}
