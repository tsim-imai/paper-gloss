use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// LaTeX text chunker
pub struct LatexChunker {
    min_words: usize,
    max_words: usize,
}

#[derive(Debug, Clone)]
pub struct LatexChunk {
    pub index: usize,
    pub text: String,
    pub content_hash: String,
    pub token_count: Option<usize>,
}

impl Default for LatexChunker {
    fn default() -> Self {
        Self {
            min_words: 800,
            max_words: 1200,
        }
    }
}

impl LatexChunker {
    /// Read and preprocess LaTeX file
    /// Removes comments, preserves math expressions
    pub fn read_latex_file(path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)
            .context("Failed to read LaTeX file")?;

        Ok(Self::preprocess_latex(&content))
    }

    /// Preprocess LaTeX: remove comments, normalize spaces
    fn preprocess_latex(content: &str) -> String {
        let mut result = String::new();
        let mut in_comment = false;

        for line in content.lines() {
            // Remove comments (lines starting with %)
            let trimmed = line.trim();
            if trimmed.starts_with('%') {
                continue;
            }

            // Remove inline comments (text after % not in math mode)
            // Simple heuristic: if line contains %, split and take before
            let cleaned = if let Some(comment_pos) = line.find('%') {
                // Check if % is escaped
                if comment_pos > 0 && line.chars().nth(comment_pos - 1) == Some('\\') {
                    line.to_string()
                } else {
                    line[..comment_pos].to_string()
                }
            } else {
                line.to_string()
            };

            result.push_str(&cleaned);
            result.push('\n');
        }

        result
    }

    /// Extract main text content from LaTeX
    /// Skips preamble, focuses on \begin{document}...\end{document}
    pub fn extract_document_body(latex: &str) -> String {
        // Find \begin{document} and \end{document}
        if let Some(begin_pos) = latex.find(r"\begin{document}") {
            let after_begin = &latex[begin_pos + 16..]; // 16 = length of "\begin{document}"

            if let Some(end_pos) = after_begin.find(r"\end{document}") {
                return after_begin[..end_pos].to_string();
            }

            // If no \end{document}, return everything after \begin{document}
            return after_begin.to_string();
        }

        // If no \begin{document}, return original
        warn!("No \\begin{{document}} found, using entire file");
        latex.to_string()
    }

    /// Remove LaTeX commands that shouldn't be translated
    /// Keeps: text, math, \section{}, environment content (removes only figure/table)
    /// Removes: \label{}, \ref{}, \cite{}, figures, tables (content skipped entirely)
    /// NOTE: Does NOT remove \begin{}/\end{} commands - those are handled at render time for flexibility
    fn strip_non_translatable(text: &str) -> String {
        use regex::Regex;

        let mut result = text.to_string();

        // Remove \label{...}
        let re_label = Regex::new(r"\\label\{[^}]*\}").unwrap();
        result = re_label.replace_all(&result, "").to_string();

        // Remove \ref{...}, \cref{...}, \Cref{...}
        let re_ref = Regex::new(r"\\[Cc]?ref\{[^}]*\}").unwrap();
        result = re_ref.replace_all(&result, "").to_string();

        // Remove \cite{...}, \citep{...}, \citet{...}
        let re_cite = Regex::new(r"\\cite[tp]?\{[^}]*\}").unwrap();
        result = re_cite.replace_all(&result, "").to_string();

        // Replace figure/table environments with their captions (if any), otherwise remove
        fn replace_env_with_captions(mut s: String, env: &str) -> String {
            let pattern = format!(r"(?s)\\begin\{{{}\*?\}}(.*?)\\end\{{{}\*?\}}", env, env);
            let re_env = Regex::new(&pattern).unwrap();
            let re_caption = Regex::new(r"\\caption\{([^}]*)\}").unwrap();
            loop {
                let caps = match re_env.captures(&s) { Some(c) => c, None => break };
                let full = caps.get(0).unwrap();
                let body = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let mut captions: Vec<String> = Vec::new();
                for cap in re_caption.captures_iter(body) {
                    if let Some(m) = cap.get(1) { captions.push(m.as_str().to_string()); }
                }
                let replacement = if captions.is_empty() { String::new() } else { format!("{}\n\n", captions.join("\n\n")) };
                s.replace_range(full.range(), &replacement);
            }
            s
        }

        result = replace_env_with_captions(result, "figure");
        result = replace_env_with_captions(result, "table");

        result
    }

    /// Chunk LaTeX text into translatable segments
    pub fn chunk(&self, latex_content: &str) -> Vec<LatexChunk> {
        let body = Self::extract_document_body(latex_content);
        let cleaned = Self::strip_non_translatable(&body);

        // Split by sections
        let sections = self.split_by_sections(&cleaned);

        let mut chunks = Vec::new();
        let mut chunk_index = 0;

        for section in sections {
            // Further split large sections by paragraphs
            let section_chunks = self.chunk_section(&section, chunk_index);
            chunk_index += section_chunks.len();
            chunks.extend(section_chunks);
        }

        chunks
    }

    /// Split text by LaTeX sections
    fn split_by_sections(&self, text: &str) -> Vec<String> {
        use regex::Regex;

        // Split on \section, \subsection, \subsubsection
        let re_section = Regex::new(r"(\\(?:sub)*section\{[^}]*\})").unwrap();

        let mut sections = Vec::new();
        let mut current = String::new();

        for line in text.lines() {
            if re_section.is_match(line) {
                if !current.trim().is_empty() {
                    sections.push(current.clone());
                }
                current = String::new();
            }
            current.push_str(line);
            current.push('\n');
        }

        if !current.trim().is_empty() {
            sections.push(current);
        }

        if sections.is_empty() {
            vec![text.to_string()]
        } else {
            sections
        }
    }

    /// Chunk a section into smaller pieces
    fn chunk_section(&self, section: &str, start_index: usize) -> Vec<LatexChunk> {
        // Split by double newlines (paragraphs)
        let paragraphs: Vec<&str> = section
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut word_count = 0;

        for para in paragraphs {
            let para_words = Self::count_words(para);

            // If adding this para would exceed max, finalize only at a safe boundary
            if word_count + para_words > self.max_words && word_count >= self.min_words {
                if !current_chunk.trim().is_empty() && Self::is_safe_boundary(&current_chunk) {
                    chunks.push(self.create_chunk(start_index + chunks.len(), &current_chunk));
                    current_chunk = String::new();
                    word_count = 0;
                }
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(para);
            word_count += para_words;
        }

        // Add final chunk
        if !current_chunk.trim().is_empty() {
            chunks.push(self.create_chunk(start_index + chunks.len(), &current_chunk));
        }

        chunks
    }

    /// Returns true if text ends at a safe LaTeX boundary (not inside math/env)
    fn is_safe_boundary(text: &str) -> bool {
        use regex::Regex;
        // Strip display and bracket/paren math, then check inline $ parity
        let mut tmp = text.to_string();
        let re_display_double = Regex::new(r"\$\$[^$]*?\$\$").unwrap();
        tmp = re_display_double.replace_all(&tmp, " ").to_string();
        let re_bracket = Regex::new(r"\\\[.*?\\\]").unwrap();
        tmp = re_bracket.replace_all(&tmp, " ").to_string();
        let re_paren = Regex::new(r"\\\(.*?\\\)").unwrap();
        tmp = re_paren.replace_all(&tmp, " ").to_string();

        let mut unescaped_dollars = 0usize;
        let bytes = tmp.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c == '$' {
                if i == 0 || bytes[i - 1] as char != '\\' { unescaped_dollars += 1; }
            }
            i += 1;
        }
        if unescaped_dollars % 2 != 0 { return false; }

        let begin_count = Regex::new(r"\\begin\{").unwrap().find_iter(text).count();
        let end_count   = Regex::new(r"\\end\{").unwrap().find_iter(text).count();
        if begin_count != end_count { return false; }

        true
    }

    /// Count words (ignoring LaTeX commands in math mode)
    fn count_words(text: &str) -> usize {
        // Simple approximation: count whitespace-separated tokens
        // that are not LaTeX commands
        text.split_whitespace()
            .filter(|w| !w.starts_with('\\') || w.len() > 10) // Keep long commands as they might be text
            .count()
    }

    /// Create chunk with hash and metadata
    fn create_chunk(&self, index: usize, text: &str) -> LatexChunk {
        let content_hash = Self::compute_hash(text);
        let token_count = Some(Self::estimate_tokens(text));

        LatexChunk {
            index,
            text: text.to_string(),
            content_hash,
            token_count,
        }
    }

    /// Compute SHA-256 hash of text
    fn compute_hash(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Estimate token count (rough: 1 token ≈ 4 characters)
    fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_latex() {
        let input = r#"
% This is a comment
Some text here
More text % inline comment
\\% This is a literal percent
"#;

        let output = LatexChunker::preprocess_latex(input);
        assert!(output.contains("Some text here"));
        assert!(output.contains("More text"));
        assert!(!output.contains("This is a comment"));
    }

    #[test]
    fn test_extract_document_body() {
        let input = r#"
\documentclass{article}
\usepackage{amsmath}
\begin{document}
This is the body.
More text here.
\end{document}
"#;

        let body = LatexChunker::extract_document_body(input);
        assert!(body.contains("This is the body"));
        assert!(!body.contains("\\documentclass"));
    }

    #[test]
    fn test_count_words() {
        assert_eq!(LatexChunker::count_words("Hello world"), 2);
        assert_eq!(LatexChunker::count_words("We have $x = 5$ here"), 5); // Rough count
    }
}
