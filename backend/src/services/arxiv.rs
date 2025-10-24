use anyhow::{Context, Result};
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// arXiv source downloader and extractor
pub struct ArxivDownloader {
    client: Client,
    storage_dir: PathBuf,
}

impl ArxivDownloader {
    pub fn new(storage_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&storage_dir)?;
        Ok(Self {
            client: Client::new(),
            storage_dir,
        })
    }

    /// Extract arXiv ID from URL
    /// Examples:
    /// - https://arxiv.org/abs/2212.14578 -> 2212.14578
    /// - https://arxiv.org/abs/2212.14578v2 -> 2212.14578v2
    pub fn extract_arxiv_id(url: &str) -> Result<String> {
        let url_lower = url.to_lowercase();

        if let Some(abs_pos) = url_lower.find("/abs/") {
            let start = abs_pos + 5; // length of "/abs/"
            let remaining = &url[start..];

            // Extract ID (stop at query params or fragment)
            let id = remaining
                .split(&['?', '#'][..])
                .next()
                .unwrap_or(remaining)
                .trim()
                .to_string();

            if id.is_empty() {
                anyhow::bail!("Invalid arXiv URL: no ID found");
            }

            Ok(id)
        } else {
            anyhow::bail!("Invalid arXiv URL: must contain /abs/");
        }
    }

    /// Download and extract arXiv source
    /// Returns the path to the extracted directory
    pub async fn download_and_extract(&self, arxiv_id: &str) -> Result<PathBuf> {
        info!("Downloading arXiv source for {}", arxiv_id);

        // Download source (.tar.gz)
        let source_url = format!("https://arxiv.org/e-print/{}", arxiv_id);
        let response = self.client
            .get(&source_url)
            .send()
            .await
            .context("Failed to download arXiv source")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download arXiv source: HTTP {}", response.status());
        }

        let bytes = response.bytes().await?;

        // Save to temp file
        let tar_path = self.storage_dir.join(format!("{}.tar.gz", arxiv_id));
        fs::write(&tar_path, &bytes)?;
        info!("Downloaded {} bytes to {:?}", bytes.len(), tar_path);

        // Extract to directory
        let extract_dir = self.storage_dir.join(format!("arXiv-{}", arxiv_id));
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir)?;
        }
        fs::create_dir_all(&extract_dir)?;

        self.extract_tar_gz(&tar_path, &extract_dir)?;

        // Remove tar file
        fs::remove_file(&tar_path)?;

        info!("Extracted arXiv source to {:?}", extract_dir);
        Ok(extract_dir)
    }

    /// Extract .tar.gz file to directory
    fn extract_tar_gz(&self, tar_path: &Path, extract_dir: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let tar_file = fs::File::open(tar_path)?;
        let decoder = GzDecoder::new(tar_file);
        let mut archive = Archive::new(decoder);

        archive.unpack(extract_dir)?;

        Ok(())
    }

    /// Find main .tex file in extracted directory
    /// Looks for: main.tex, paper.tex, or the largest .tex file
    pub fn find_main_tex(&self, extract_dir: &Path) -> Result<PathBuf> {
        // Try common names first
        for name in &["main.tex", "paper.tex", "manuscript.tex"] {
            let path = extract_dir.join(name);
            if path.exists() {
                info!("Found main TeX file: {:?}", path);
                return Ok(path);
            }
        }

        // Find all .tex files
        let mut tex_files: Vec<(PathBuf, u64)> = Vec::new();

        for entry in fs::read_dir(extract_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("tex") {
                if let Ok(metadata) = fs::metadata(&path) {
                    tex_files.push((path, metadata.len()));
                }
            }
        }

        if tex_files.is_empty() {
            anyhow::bail!("No .tex files found in extracted arXiv source");
        }

        // Return largest .tex file
        tex_files.sort_by_key(|(_, size)| std::cmp::Reverse(*size));
        let (largest_tex, size) = &tex_files[0];

        warn!("No main.tex found, using largest .tex file: {:?} ({} bytes)", largest_tex, size);
        Ok(largest_tex.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_arxiv_id() {
        assert_eq!(
            ArxivDownloader::extract_arxiv_id("https://arxiv.org/abs/2212.14578").unwrap(),
            "2212.14578"
        );

        assert_eq!(
            ArxivDownloader::extract_arxiv_id("https://arxiv.org/abs/2212.14578v2").unwrap(),
            "2212.14578v2"
        );

        assert_eq!(
            ArxivDownloader::extract_arxiv_id("https://arxiv.org/abs/1234.5678?foo=bar").unwrap(),
            "1234.5678"
        );
    }

    #[test]
    fn test_invalid_url() {
        assert!(ArxivDownloader::extract_arxiv_id("https://example.com").is_err());
        assert!(ArxivDownloader::extract_arxiv_id("https://arxiv.org/pdf/123.pdf").is_err());
    }
}
