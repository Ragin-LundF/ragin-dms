//! What the AI (or the matching rules) suggest for a document. Suggestions are values, never applied state.

pub mod model;
mod suggested_category;

pub use model::{ClassificationSuggestion, EInvoicePart, RequestedAction, SuggestedFolder, SuggestedTag};
pub use suggested_category::SuggestedCategory;
