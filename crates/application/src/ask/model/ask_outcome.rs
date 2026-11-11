use crate::ask::Citation;
use crate::port::ai::ClassificationFailure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskOutcome {
    /// The answer from `model` with the sources it cites; `used_vectors` is `false` when only words were searched.
    Answered {
        text: String,
        citations: Vec<Citation>,
        model: String,
        used_vectors: bool,
    },
    /// Nothing in the library fits the question; the AI was not asked to make something up.
    NotFound,
    LibraryNotOpen,
    Failed {
        reason: ClassificationFailure,
        detail: Option<String>,
    },
}
