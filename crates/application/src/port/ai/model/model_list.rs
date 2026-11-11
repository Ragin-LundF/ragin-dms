use crate::port::ai::ClassificationFailure;

/// The answer of an AI server to "which models do you have?".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelList {
    Available(Vec<String>),
    Unavailable { reason: ClassificationFailure, detail: Option<String> },
}
