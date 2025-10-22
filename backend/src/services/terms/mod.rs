// Term-related services
pub mod extraction;
pub mod normalization;
pub mod definition;
pub mod occurrence_tracker;
pub mod search;
pub mod duplicate_detection;
pub mod merge;
pub mod tagger;

pub use extraction::{TermExtractor, ExtractedTerm};
pub use normalization::{
    normalize_english, normalize_japanese,
    generate_english_variants, generate_japanese_variants,
};
pub use definition::DefinitionGenerator;
pub use occurrence_tracker::OccurrenceTracker;
pub use search::{TermSearchService, TermSearchResult};
pub use duplicate_detection::{DuplicateDetectionService, DuplicatePair};
pub use merge::TermMergeService;
pub use tagger::{TermTagger, TaggingResult, TagInfo, strip_sentinels, parse_tagged_spans};
