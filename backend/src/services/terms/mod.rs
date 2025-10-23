// Term-related services
pub mod normalization;
pub mod definition;
pub mod jp_extraction;
pub mod scanner_ja;
pub mod search;
pub mod duplicate_detection;
pub mod merge;

pub use normalization::{
    normalize_japanese,
    generate_english_variants,
    generate_japanese_variants,
};
pub use definition::DefinitionGenerator;
pub use search::{TermSearchService, TermSearchResult};
pub use duplicate_detection::DuplicateDetectionService;
pub use merge::TermMergeService;
pub use jp_extraction::JapaneseTermExtractor;
pub use scanner_ja::OccurrenceScannerJa;
