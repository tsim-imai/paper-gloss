/// Term normalization utilities (data-model.md:294-322)
/// Handles English and Japanese variant matching

/// Normalize English term for matching
/// - Lowercase conversion
/// - Hyphen/space normalization
/// - Basic stemming (future)
pub fn normalize_english(term: &str) -> String {
    term.to_lowercase()
        .replace('-', " ")
        .replace('_', " ")
        .trim()
        .to_string()
}

/// Normalize Japanese term for matching
/// - Full-width to half-width katakana
/// - Remove middle dot (・)
/// - Normalize long vowel mark (ー)
pub fn normalize_japanese(term: &str) -> String {
    term.chars()
        .map(|c| match c {
            // Remove middle dot
            '・' | '･' => ' ',
            // Normalize long vowel marks (keep as-is for now)
            'ー' => 'ー',
            // Full-width to half-width conversion (simplified)
            // TODO: Complete katakana conversion table
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Generate common English variants
pub fn generate_english_variants(term: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let normalized = normalize_english(term);

    // Original
    variants.push(term.to_string());

    // Normalized
    if normalized != term {
        variants.push(normalized.clone());
    }

    // With hyphens
    let hyphenated = normalized.replace(' ', "-");
    if hyphenated != normalized {
        variants.push(hyphenated);
    }

    // With underscores
    let underscored = normalized.replace(' ', "_");
    if underscored != normalized {
        variants.push(underscored);
    }

    // Lowercase
    let lowercase = term.to_lowercase();
    if lowercase != term {
        variants.push(lowercase);
    }

    // Remove duplicates
    variants.sort();
    variants.dedup();

    variants
}

/// Generate common Japanese variants
pub fn generate_japanese_variants(term: &str) -> Vec<String> {
    let mut variants = Vec::new();

    // Original
    variants.push(term.to_string());

    // Normalized
    let normalized = normalize_japanese(term);
    if normalized != term {
        variants.push(normalized);
    }

    // Remove duplicates
    variants.sort();
    variants.dedup();

    variants
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_english() {
        assert_eq!(normalize_english("neural-network"), "neural network");
        assert_eq!(normalize_english("Neural_Network"), "neural network");
        assert_eq!(normalize_english("NEURAL NETWORK"), "neural network");
    }

    #[test]
    fn test_normalize_japanese() {
        assert_eq!(normalize_japanese("ニューラル・ネットワーク"), "ニューラル ネットワーク");
        assert_eq!(normalize_japanese("ニューラルネットワーク"), "ニューラルネットワーク");
    }

    #[test]
    fn test_generate_english_variants() {
        let variants = generate_english_variants("neural network");
        assert!(variants.contains(&"neural network".to_string()));
        assert!(variants.contains(&"neural-network".to_string()));
        assert!(variants.contains(&"neural_network".to_string()));
    }
}
