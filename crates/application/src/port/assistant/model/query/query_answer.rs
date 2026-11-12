use crate::port::ai::ClassificationFailure;
use crate::port::assistant::TranslatedQuery;

/// The AI's reading of a sentence, from `model`, or why there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryAnswer {
    Translated { query: TranslatedQuery, model: String },
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
