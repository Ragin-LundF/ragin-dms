//! Searching documents by their text (`ContentMatcher`, `SearchContents`, `FindDocuments`) and the smart folders a
//! library starts with.

mod content_matcher;
mod find_documents;
pub mod model;
mod saved_search_presets;
mod search_contents;

pub use content_matcher::ContentMatcher;
pub use find_documents::FindDocuments;
pub use model::{FindOutcome, IndexedText, SearchOutcome};
pub use saved_search_presets::SavedSearchPresets;
pub use search_contents::SearchContents;
