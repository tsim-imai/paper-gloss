// Term-related services
pub mod extraction;
pub mod normalization;
pub mod definition;
pub mod occurrence_tracker;

pub use extraction::{TermExtractor, ExtractedTerm};
pub use normalization::{
    normalize_english, normalize_japanese,
    generate_english_variants, generate_japanese_variants,
};
pub use definition::DefinitionGenerator;
pub use occurrence_tracker::OccurrenceTracker;
