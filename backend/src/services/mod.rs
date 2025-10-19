// Business logic services
pub mod llm;
pub mod pdf;
pub mod translation;
pub mod paper_processor;

pub use translation::TranslationService;
pub use paper_processor::PaperProcessor;
