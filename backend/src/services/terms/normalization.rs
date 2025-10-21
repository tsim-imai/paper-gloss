/// Term normalization utilities (data-model.md:294-322)
/// Handles English and Japanese variant matching

/// Normalize English term for matching
/// - Lowercase conversion
/// - Hyphen/space normalization
/// - Basic stemming (simple pluralization handling)
pub fn normalize_english(term: &str) -> String {
    let lowercased = term.to_lowercase()
        .replace('-', " ")
        .replace('_', " ");

    // Collapse multiple spaces and trim
    let normalized = lowercased
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    // Apply simple pluralization rules to each word
    let words: Vec<String> = normalized
        .split(' ')
        .map(|word| simple_singularize(word))
        .collect();

    words.join(" ")
}

/// Simple singularization for common cases
/// Not comprehensive, but covers common ML terms
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn simple_singularize(word: &str) -> String {
    // Don't singularize short words or acronyms
    if word.len() <= 3 || word.chars().all(|c| c.is_uppercase()) {
        return word.to_string();
    }

    // Common irregular plurals in ML
    match word {
        "losses" => "loss".to_string(),
        "biases" => "bias".to_string(),
        "matrices" => "matrix".to_string(),
        "indices" => "index".to_string(),
        _ => {
            // Simple rules
            if word.ends_with("ies") && word.len() > 4 {
                // policies -> policy, strategies -> strategy
                format!("{}y", &word[..word.len() - 3])
            } else if word.ends_with("ses") || word.ends_with("xes") || word.ends_with("zes") {
                // losses -> loss, boxes -> box, sizes -> size
                word[..word.len() - 2].to_string()
            } else if word.ends_with("s")
                && !word.ends_with("ss")  // class, loss
                && !word.ends_with("us")  // corpus
                && !word.ends_with("is")  // basis, analysis
                && !word.ends_with("as")  // bias, alias
            {
                // models -> model, networks -> network
                word[..word.len() - 1].to_string()
            } else {
                word.to_string()
            }
        }
    }
}

/// Normalize Japanese term for matching
/// - Full-width to half-width katakana
/// - Remove middle dot (・)
/// - Normalize long vowel mark (ー)
pub fn normalize_japanese(term: &str) -> String {
    let normalized = term.chars()
        .map(|c| match c {
            // Remove middle dot (both full-width and half-width)
            '・' | '･' => ' ',
            // Normalize long vowel marks (keep as-is for now)
            'ー' => 'ー',
            // Half-width katakana to full-width conversion
            'ｱ' => 'ア', 'ｲ' => 'イ', 'ｳ' => 'ウ', 'ｴ' => 'エ', 'ｵ' => 'オ',
            'ｶ' => 'カ', 'ｷ' => 'キ', 'ｸ' => 'ク', 'ｹ' => 'ケ', 'ｺ' => 'コ',
            'ｻ' => 'サ', 'ｼ' => 'シ', 'ｽ' => 'ス', 'ｾ' => 'セ', 'ｿ' => 'ソ',
            'ﾀ' => 'タ', 'ﾁ' => 'チ', 'ﾂ' => 'ツ', 'ﾃ' => 'テ', 'ﾄ' => 'ト',
            'ﾅ' => 'ナ', 'ﾆ' => 'ニ', 'ﾇ' => 'ヌ', 'ﾈ' => 'ネ', 'ﾉ' => 'ノ',
            'ﾊ' => 'ハ', 'ﾋ' => 'ヒ', 'ﾌ' => 'フ', 'ﾍ' => 'ヘ', 'ﾎ' => 'ホ',
            'ﾏ' => 'マ', 'ﾐ' => 'ミ', 'ﾑ' => 'ム', 'ﾒ' => 'メ', 'ﾓ' => 'モ',
            'ﾔ' => 'ヤ', 'ﾕ' => 'ユ', 'ﾖ' => 'ヨ',
            'ﾗ' => 'ラ', 'ﾘ' => 'リ', 'ﾙ' => 'ル', 'ﾚ' => 'レ', 'ﾛ' => 'ロ',
            'ﾜ' => 'ワ', 'ｦ' => 'ヲ', 'ﾝ' => 'ン',
            'ｧ' => 'ァ', 'ｨ' => 'ィ', 'ｩ' => 'ゥ', 'ｪ' => 'ェ', 'ｫ' => 'ォ',
            'ｬ' => 'ャ', 'ｭ' => 'ュ', 'ｮ' => 'ョ', 'ｯ' => 'ッ',
            'ｰ' => 'ー', // Half-width long vowel to full-width
            'ﾞ' => '゛', 'ﾟ' => '゜', // Dakuten and handakuten
            // Keep other characters as-is
            _ => c,
        })
        .collect::<String>();

    // Collapse multiple spaces into single space and trim
    normalized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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

    // Normalized (with spaces for middle dots) - ALWAYS add this
    let normalized = normalize_japanese(term);
    if normalized != term {
        variants.push(normalized.clone());
    }

    // Without any spaces (completely concatenated)
    let no_spaces = normalized.replace(' ', "");
    if no_spaces != term && no_spaces != normalized {
        variants.push(no_spaces.clone());
    }

    // With middle dots (if the original doesn't have them)
    if !term.contains('・') && !term.contains('･') {
        // Try to intelligently add middle dots for compound katakana words
        // This is a heuristic approach for common patterns
        if term.contains("ネットワーク") {
            let with_dots = term.replace("ネットワーク", "・ネットワーク");
            if with_dots != term {
                variants.push(with_dots.clone());
            }

            // Also add the normalized version of the dotted form
            let normalized_dots = normalize_japanese(&with_dots);
            if normalized_dots != with_dots && !variants.contains(&normalized_dots) {
                variants.push(normalized_dots);
            }
        }
        // Add more patterns as needed
    }

    // If original has middle dots, also add version without them
    if term.contains('・') || term.contains('･') {
        let without_dots = term.replace('・', "").replace('･', "");
        if without_dots != term {
            variants.push(without_dots);
        }
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
        // Test multiple spaces
        assert_eq!(normalize_english("neural   network"), "neural network");
        // Test pluralization
        assert_eq!(normalize_english("neural-networks"), "neural network");
        assert_eq!(normalize_english("models"), "model");
        assert_eq!(normalize_english("losses"), "loss");
        assert_eq!(normalize_english("policies"), "policy");
        assert_eq!(normalize_english("biases"), "bias");
    }

    #[test]
    fn test_normalize_japanese() {
        assert_eq!(normalize_japanese("ニューラル・ネットワーク"), "ニューラル ネットワーク");
        assert_eq!(normalize_japanese("ニューラルネットワーク"), "ニューラルネットワーク");
        // Test half-width katakana conversion
        assert_eq!(normalize_japanese("ﾆｭｰﾗﾙﾈｯﾄﾜｰｸ"), "ニューラルネットワーク");
        // Test mixed
        assert_eq!(normalize_japanese("ニューラル・ﾈｯﾄﾜｰｸ"), "ニューラル ネットワーク");
    }

    #[test]
    fn test_generate_english_variants() {
        let variants = generate_english_variants("neural network");
        assert!(variants.contains(&"neural network".to_string()));
        assert!(variants.contains(&"neural-network".to_string()));
        assert!(variants.contains(&"neural_network".to_string()));
    }

    #[test]
    fn test_english_pluralization() {
        // Test that pluralized forms normalize to singular
        assert_eq!(normalize_english("networks"), "network");
        assert_eq!(normalize_english("models"), "model");
        assert_eq!(normalize_english("losses"), "loss");
        assert_eq!(normalize_english("policies"), "policy");
        assert_eq!(normalize_english("matrices"), "matrix");
        assert_eq!(normalize_english("indices"), "index");
        // Words that shouldn't change
        assert_eq!(normalize_english("loss"), "loss");
        assert_eq!(normalize_english("class"), "class");
        assert_eq!(normalize_english("bias"), "bias");
        assert_eq!(normalize_english("nn"), "nn");  // Too short
        assert_eq!(normalize_english("corpus"), "corpus");  // ends with 'us'
    }

    #[test]
    fn test_generate_japanese_variants() {
        // Test with original without middle dot
        let variants1 = generate_japanese_variants("ニューラルネットワーク");
        assert!(variants1.contains(&"ニューラルネットワーク".to_string()));
        assert!(variants1.contains(&"ニューラル・ネットワーク".to_string()));

        // Test with original with middle dot
        let variants2 = generate_japanese_variants("ニューラル・ネットワーク");
        assert!(variants2.contains(&"ニューラル・ネットワーク".to_string()));
        assert!(variants2.contains(&"ニューラルネットワーク".to_string()));
        assert!(variants2.contains(&"ニューラル ネットワーク".to_string()));

        // Test half-width middle dot
        let variants3 = generate_japanese_variants("ニューラル･ネットワーク");
        assert!(variants3.contains(&"ニューラルネットワーク".to_string()));
    }
}
