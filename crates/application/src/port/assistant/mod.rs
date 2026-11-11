//! The AI reading one document, or answering about the whole library.

mod document_assistant;
mod library_assistant;
pub mod model;

pub use document_assistant::DocumentAssistant;
pub use library_assistant::LibraryAssistant;
pub use model::{AssistantAnswer, AssistantRequest, ChatRole, ChatTurn, SourceChunk};
