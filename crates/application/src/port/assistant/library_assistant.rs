use crate::port::assistant::{AssistantAnswer, SourceChunk};

/// Answers a question about the whole library from the passages found for it, citing them as `[n]`, and saying so
/// plainly when they do not hold the answer.
pub trait LibraryAssistant: Send + Sync {
    fn answer(&self, question: &str, sources: &[SourceChunk]) -> AssistantAnswer;
}
