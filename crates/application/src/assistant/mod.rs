//! The document assistant: summary, tags and questions about one document.

mod ask_about_document;
mod assistant_outcomes;
mod document_reading;
pub mod model;
mod suggest_tags;
mod summarize_document;
mod tag_all_untagged;

pub use ask_about_document::AskAboutDocument;
pub use assistant_outcomes::AssistantOutcomes;
pub use document_reading::DocumentReading;
pub use model::{AnswerOutcome, ReadingResult};
pub use suggest_tags::SuggestTags;
pub use summarize_document::SummarizeDocument;
pub use tag_all_untagged::TagAllUntagged;
