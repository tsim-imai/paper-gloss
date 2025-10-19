use sha2::{Digest, Sha256};

/// Text chunking service (FR-008, FR-009, FR-010)
pub struct TextChunker {
    min_words: usize,
    max_words: usize,
    overlap_percentage: f32,
}

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub index: usize,
    pub text: String,
    pub content_hash: String,
    pub token_count: Option<usize>,
}

impl Default for TextChunker {
    fn default() -> Self {
        Self {
            min_words: 800,
            max_words: 1200,
            overlap_percentage: 0.12, // 12% overlap (10-15% range)
        }
    }
}

impl TextChunker {
    /// Create a new text chunker with custom parameters
    pub fn new(min_words: usize, max_words: usize, overlap_percentage: f32) -> Self {
        Self {
            min_words,
            max_words,
            overlap_percentage,
        }
    }

    /// Chunk text into overlapping segments
    pub fn chunk(&self, text: &str) -> Vec<TextChunk> {
        let paragraphs = self.split_into_paragraphs(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut word_count = 0;
        let mut chunk_index = 0;

        for paragraph in paragraphs {
            let para_words = Self::count_words(&paragraph);

            // If adding this paragraph would exceed max_words, finalize current chunk
            if word_count + para_words > self.max_words && word_count >= self.min_words {
                chunks.push(self.create_chunk(chunk_index, &current_chunk));
                chunk_index += 1;

                // Apply overlap: keep last N% of words
                let overlap_words = (word_count as f32 * self.overlap_percentage) as usize;
                current_chunk = Self::get_last_n_words(&current_chunk, overlap_words);
                word_count = overlap_words;
            }

            // Add paragraph to current chunk
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(&paragraph);
            word_count += para_words;

            // If we've reached min_words and this is a natural boundary, we could chunk here
            // but we'll continue to max_words for consistency
        }

        // Add final chunk if not empty
        if !current_chunk.trim().is_empty() {
            chunks.push(self.create_chunk(chunk_index, &current_chunk));
        }

        chunks
    }

    /// Split text into paragraphs (natural boundaries)
    fn split_into_paragraphs(&self, text: &str) -> Vec<String> {
        text.split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Count words in text
    fn count_words(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Get last N words from text
    fn get_last_n_words(text: &str, n: usize) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if n >= words.len() {
            return text.to_string();
        }

        words[words.len() - n..].join(" ")
    }

    /// Create a chunk with hash and metadata
    fn create_chunk(&self, index: usize, text: &str) -> TextChunk {
        let content_hash = Self::compute_hash(text);
        let token_count = Some(Self::estimate_tokens(text));

        TextChunk {
            index,
            text: text.to_string(),
            content_hash,
            token_count,
        }
    }

    /// Compute SHA-256 hash of text (FR-010: caching)
    fn compute_hash(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Estimate token count (rough approximation: 1 token ≈ 4 characters)
    fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_small_text() {
        let chunker = TextChunker::default();
        let text = "This is a short paragraph.\n\nAnd another one.";
        let chunks = chunker.chunk(text);

        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content_hash.len() > 0);
    }

    #[test]
    fn test_chunk_large_text() {
        let chunker = TextChunker::new(10, 20, 0.1);
        let paragraph = "word ".repeat(25);
        let text = format!("{}\n\n{}", paragraph, paragraph);

        let chunks = chunker.chunk(&text);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_count_words() {
        assert_eq!(TextChunker::count_words("hello world"), 2);
        assert_eq!(TextChunker::count_words("  spaces   everywhere  "), 2);
    }

    #[test]
    fn test_compute_hash() {
        let hash1 = TextChunker::compute_hash("test");
        let hash2 = TextChunker::compute_hash("test");
        let hash3 = TextChunker::compute_hash("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA-256 hex length
    }
}
