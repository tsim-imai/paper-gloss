use anyhow::{Context, Result};
use pdf_extract::extract_text;
use std::fs;
use std::path::Path;
use tracing::{debug, error, warn};

/// PDF text extraction service (FR-006, FR-007)
pub struct PdfExtractor;

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub text: String,
    pub page_count: usize,
    pub failed_pages: Vec<usize>,
    pub warnings: Vec<String>,
    pub is_partial: bool,
}

#[derive(Debug)]
pub enum PdfError {
    FileNotFound,
    InvalidFormat,
    CorruptedFile,
    EmptyFile,
    UnsupportedFeatures,
    PartialExtraction { extracted_pages: usize, total_pages: usize },
}

impl PdfExtractor {
    /// Maximum file size in bytes (100MB)
    const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

    /// Minimum text length to consider extraction successful
    const MIN_TEXT_LENGTH: usize = 100;

    /// Extract text from a PDF file
    pub fn extract_from_file(file_path: &Path) -> Result<ExtractionResult> {
        debug!("Extracting text from PDF: {:?}", file_path);

        // Pre-validation
        Self::validate_file(file_path)?;

        // Extract text using pdf-extract
        let text = extract_text(file_path)
            .context("Failed to extract text from PDF")?;

        // Basic validation
        let mut warnings = Vec::new();
        if text.trim().is_empty() {
            warn!("Extracted text is empty from {:?}", file_path);
            warnings.push("PDF appears to be empty or may contain only images".to_string());
        }

        // Simple page count estimation (not accurate without full PDF parsing)
        let estimated_pages = text.lines().count() / 50; // Rough estimate
        let page_count = estimated_pages.max(1);

        debug!(
            "Extracted {} characters from approximately {} pages",
            text.len(),
            page_count
        );

        Ok(ExtractionResult {
            text,
            page_count,
            failed_pages: Vec::new(),
            warnings,
            is_partial: false,
        })
    }

    /// Validate file before extraction (FR-005)
    fn validate_file(file_path: &Path) -> Result<()> {
        // Check if file exists
        if !file_path.exists() {
            anyhow::bail!("PDF file does not exist: {:?}", file_path);
        }

        // Check file size
        let metadata = fs::metadata(file_path)?;
        if metadata.len() == 0 {
            anyhow::bail!("PDF file is empty (0 bytes)");
        }
        if metadata.len() > Self::MAX_FILE_SIZE {
            anyhow::bail!(
                "PDF file is too large: {} MB (max: {} MB)",
                metadata.len() / (1024 * 1024),
                Self::MAX_FILE_SIZE / (1024 * 1024)
            );
        }

        // Check file extension
        if file_path.extension().and_then(|s| s.to_str()) != Some("pdf") {
            warn!("File may not be a PDF (no .pdf extension): {:?}", file_path);
        }

        // Check PDF header (first 5 bytes should be "%PDF-")
        let mut file = fs::File::open(file_path)?;
        let mut header = vec![0u8; 5];
        use std::io::Read;
        file.read_exact(&mut header)
            .context("Failed to read PDF header")?;

        if &header != b"%PDF-" {
            anyhow::bail!("File is not a valid PDF (invalid header)");
        }

        Ok(())
    }

    /// Validate if a PDF file can be processed (basic checks)
    pub fn validate_pdf(file_path: &Path) -> Result<bool> {
        // Use the new validation method
        if let Err(e) = Self::validate_file(file_path) {
            warn!("PDF validation failed: {}", e);
            return Ok(false);
        }

        // Try to extract a small amount to verify it's readable
        match extract_text(file_path) {
            Ok(text) => {
                if text.trim().len() < Self::MIN_TEXT_LENGTH {
                    warn!("PDF has insufficient extractable text (possibly scanned)");
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            Err(e) => {
                warn!("PDF content extraction failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Extract text with error recovery (FR-007: partial processing)
    pub fn extract_with_recovery(file_path: &Path) -> Result<ExtractionResult> {
        // First try normal extraction
        match Self::extract_from_file(file_path) {
            Ok(mut result) => {
                // Check if we got meaningful content
                if result.text.trim().len() < Self::MIN_TEXT_LENGTH {
                    result.warnings.push(
                        "Extracted text is very short, PDF may be corrupted or contain mostly images".to_string()
                    );
                    result.is_partial = true;
                }
                Ok(result)
            }
            Err(e) => {
                error!("PDF extraction failed: {}", e);

                // Try to determine the type of failure
                let mut warnings = vec![format!("PDF extraction failed: {}", e)];
                let mut is_partial = false;

                // Check if file is corrupted but we can extract something
                if let Ok(partial_text) = extract_text(file_path) {
                    if !partial_text.trim().is_empty() {
                        warn!("Partial extraction successful despite errors");
                        warnings.push("Partial content extracted, some pages may be missing".to_string());
                        is_partial = true;

                        let estimated_pages = partial_text.lines().count() / 50;
                        return Ok(ExtractionResult {
                            text: partial_text,
                            page_count: estimated_pages.max(1),
                            failed_pages: vec![],  // We don't know which specific pages failed
                            warnings,
                            is_partial,
                        });
                    }
                }

                // Complete failure - return empty result but don't fail the whole process
                warnings.push("No text could be extracted from this PDF".to_string());
                warnings.push("The file may be corrupted, encrypted, or contain only scanned images".to_string());

                Ok(ExtractionResult {
                    text: String::new(),
                    page_count: 0,
                    failed_pages: vec![0], // Indicate complete failure
                    warnings,
                    is_partial: false,  // Nothing was extracted
                })
            }
        }
    }

    /// Check if extraction result is usable
    pub fn is_extraction_usable(result: &ExtractionResult) -> bool {
        !result.text.trim().is_empty() && result.text.len() >= Self::MIN_TEXT_LENGTH
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

    #[test]
    fn test_extract_with_recovery_returns_empty_on_failure() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not a valid pdf").unwrap();

        let result = PdfExtractor::extract_with_recovery(temp_file.path()).unwrap();
        assert_eq!(result.text, "");
        assert!(result.page_count == 0);
        assert!(!result.failed_pages.is_empty());
    }
}
