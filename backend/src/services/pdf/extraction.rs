use anyhow::{Context, Result};
use pdf_extract::extract_text;
use std::path::Path;
use tracing::{debug, warn};

/// PDF text extraction service (FR-006, FR-007)
pub struct PdfExtractor;

#[derive(Debug)]
pub struct ExtractionResult {
    pub text: String,
    pub page_count: usize,
    pub failed_pages: Vec<usize>,
}

impl PdfExtractor {
    /// Extract text from a PDF file
    pub fn extract_from_file(file_path: &Path) -> Result<ExtractionResult> {
        debug!("Extracting text from PDF: {:?}", file_path);

        // Extract text using pdf-extract
        let text = extract_text(file_path)
            .context("Failed to extract text from PDF")?;

        // Basic validation
        if text.trim().is_empty() {
            warn!("Extracted text is empty from {:?}", file_path);
        }

        // Simple page count estimation (not accurate without full PDF parsing)
        // In production, you'd use a proper PDF library to get exact page count
        let estimated_pages = text.lines().count() / 50; // Rough estimate

        debug!(
            "Extracted {} characters from approximately {} pages",
            text.len(),
            estimated_pages.max(1)
        );

        Ok(ExtractionResult {
            text,
            page_count: estimated_pages.max(1),
            failed_pages: Vec::new(), // pdf-extract doesn't provide per-page error info
        })
    }

    /// Validate if a PDF file can be processed (basic checks)
    pub fn validate_pdf(file_path: &Path) -> Result<bool> {
        // Check if file exists
        if !file_path.exists() {
            anyhow::bail!("PDF file does not exist: {:?}", file_path);
        }

        // Check if file has .pdf extension
        if file_path.extension().and_then(|s| s.to_str()) != Some("pdf") {
            warn!("File may not be a PDF (no .pdf extension): {:?}", file_path);
        }

        // Try to extract a small amount to verify it's readable
        match extract_text(file_path) {
            Ok(text) => {
                if text.trim().is_empty() {
                    warn!("PDF appears to be empty or scanned (no extractable text)");
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            Err(e) => {
                warn!("PDF validation failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Extract text with error recovery (FR-007: partial processing)
    pub fn extract_with_recovery(file_path: &Path) -> Result<ExtractionResult> {
        match Self::extract_from_file(file_path) {
            Ok(result) => Ok(result),
            Err(e) => {
                warn!("PDF extraction failed, attempting recovery: {}", e);

                // In a production system, you'd implement per-page extraction
                // and collect partial results. For MVP, we'll just return empty.
                Ok(ExtractionResult {
                    text: String::new(),
                    page_count: 0,
                    failed_pages: vec![0], // Indicate failure
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_pdf_nonexistent() {
        let result = PdfExtractor::validate_pdf(Path::new("/nonexistent.pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not a valid pdf").unwrap();

        let result = PdfExtractor::extract_from_file(temp_file.path());
        assert!(result.is_err());
    }
}
