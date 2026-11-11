use crate::port::ai::ClassificationFailure;

/// The text of one line read again, or why the AI could not read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineOutcome {
    Read(String),
    Failure { reason: ClassificationFailure, detail: Option<String> },
}
