use std::sync::Arc;

use dms_application::port::assistant::{AssistantAnswer, LibraryAssistant, SourceChunk};
use dms_domain::text;

use super::text_answer_of;
use crate::client::chat_request::{self, ChatMessage};
use crate::client::{ChatOptions, ChatTransport};
use crate::prompt;

const TEMPERATURE: f64 = 0.2;
/// In UTF-16 units, as Kotlin's `take`.
const MAX_PASSAGE: usize = 4_000;

/// Answers questions about the whole library from the passages found, on the chat model.
pub struct OpenAiLibraryAssistant {
    transport: Arc<ChatTransport>,
}

impl OpenAiLibraryAssistant {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }
}

impl LibraryAssistant for OpenAiLibraryAssistant {
    fn answer(&self, question: &str, sources: &[SourceChunk]) -> AssistantAnswer {
        let passages: Vec<String> = sources
            .iter()
            .map(|source| format!("[{}] {}", source.number, text::take_utf16(&source.text, MAX_PASSAGE)))
            .collect();
        let messages = [
            ChatMessage::system(&prompt::ASK_LIBRARY.fill(&[("sources", &passages.join("\n\n"))])),
            ChatMessage::user(question),
        ];
        let reply =
            self.transport.complete(ChatOptions::default(), |model| chat_request::chat(model, TEMPERATURE, &messages));
        text_answer_of(reply)
    }
}
