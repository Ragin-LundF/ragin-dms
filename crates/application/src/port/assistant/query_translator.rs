use crate::port::assistant::{QueryAnswer, QueryRequest};

/// Lets the AI turn a sentence in plain words into the parts of a library filter. It never searches itself.
pub trait QueryTranslator: Send + Sync {
    fn translate(&self, request: &QueryRequest) -> QueryAnswer;
}
