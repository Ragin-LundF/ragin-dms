//! The AI reading one document (summary, tags, questions) or answering about the whole library.

mod assistant_prompts;
mod open_ai_assistant;
mod open_ai_library_assistant;
mod tag_answer_parser;

use dms_application::port::assistant::AssistantAnswer;
pub use open_ai_assistant::OpenAiAssistant;
pub use open_ai_library_assistant::OpenAiLibraryAssistant;

use crate::client::{AnswerText, ChatReply};

/// A free-text answer without thoughts and emphasis, with the model that gave it.
fn text_answer_of(reply: ChatReply) -> AssistantAnswer {
    match reply {
        ChatReply::Answered { content, model } => AssistantAnswer::Text { text: AnswerText::plain(&content), model },
        ChatReply::Failed { reason, detail } => AssistantAnswer::Failed { reason, detail },
    }
}

#[cfg(test)]
mod tests;
