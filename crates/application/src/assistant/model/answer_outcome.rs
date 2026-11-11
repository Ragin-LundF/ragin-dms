use crate::port::ai::ClassificationFailure;

/// The outcome of a summary or a question about one document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerOutcome {
    Answered { text: String, model: String },
    LibraryNotOpen,
    NotFound,
    NotReadable { detail: Option<String> },
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
