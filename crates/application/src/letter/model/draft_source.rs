use crate::port::ai::ClassificationFailure;

/// Who wrote a draft: the AI with `model`, or the fixed template, because AI is off (`ai_failure` `None`) or the
/// AI failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftSource {
    Ai { model: String },
    Template { ai_failure: Option<ClassificationFailure> },
}
