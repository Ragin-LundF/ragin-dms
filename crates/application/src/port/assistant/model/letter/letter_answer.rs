use crate::port::ai::ClassificationFailure;
use crate::port::assistant::LetterDraft;

/// The AI's draft of a letter, from `model`, or why there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LetterAnswer {
    Drafted { draft: LetterDraft, model: String },
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
