use std::sync::Arc;

use dms_application::port::ai::{ClassificationFailure, DocumentContent, PageImage};
use dms_application::port::assistant::{AssistantAnswer, AssistantRequest, ChatRole, ChatTurn, DocumentAssistant};
use dms_domain::taxonomy::TagName;
use indexmap::IndexSet;
use serde_json::Value;

use super::{assistant_prompts, tag_answer_parser, text_answer_of};
use crate::client::chat_request::{self, ChatMessage};
use crate::client::{ChatOptions, ChatReply, ChatTransport, shortened};

const SUMMARY_TEMPERATURE: f64 = 0.2;
const TAGS_TEMPERATURE: f64 = 0.1;
const CHAT_TEMPERATURE: f64 = 0.3;

/// The document assistant on any server with the OpenAI chat completions API, or on Anthropic.
pub struct OpenAiAssistant {
    transport: Arc<ChatTransport>,
}

impl OpenAiAssistant {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }

    fn send(&self, request: &AssistantRequest, temperature: f64, messages: &[ChatMessage]) -> ChatReply {
        let has_images = matches!(request.content, DocumentContent::PageImages(_));
        let options = ChatOptions { has_images, ..ChatOptions::default() };
        self.transport.complete(options, |model| chat_request::chat(model, temperature, messages))
    }
}

impl DocumentAssistant for OpenAiAssistant {
    fn summarize(&self, request: &AssistantRequest) -> AssistantAnswer {
        let messages = [
            ChatMessage::system(&assistant_prompts::summary()),
            user(&assistant_prompts::summary_request(request), request),
        ];
        text_answer_of(self.send(request, SUMMARY_TEMPERATURE, &messages))
    }

    fn suggest_tags(&self, request: &AssistantRequest, existing_tags: &IndexSet<TagName>) -> AssistantAnswer {
        let messages = [
            ChatMessage::system(&assistant_prompts::tags(existing_tags)),
            user(&assistant_prompts::tags_request(request), request),
        ];
        match self.send(request, TAGS_TEMPERATURE, &messages) {
            ChatReply::Failed { reason, detail } => AssistantAnswer::Failed { reason, detail },
            ChatReply::Answered { content, model } => {
                let tags = tag_answer_parser::parse(&content, existing_tags);
                if tags.is_empty() {
                    AssistantAnswer::Failed {
                        reason: ClassificationFailure::InvalidResponse,
                        detail: Some(shortened(&content)),
                    }
                } else {
                    AssistantAnswer::Tags { tags, model }
                }
            }
        }
    }

    fn answer(&self, request: &AssistantRequest, conversation: &[ChatTurn]) -> AssistantAnswer {
        let first_question = conversation.iter().position(|turn| turn.role == ChatRole::User);
        let mut messages = vec![ChatMessage::system(&assistant_prompts::chat(request))];
        messages.extend(conversation.iter().enumerate().map(|(index, turn)| {
            if Some(index) == first_question {
                user(&turn.text, request)
            } else {
                let role = if turn.role == ChatRole::Assistant { "assistant" } else { "user" };
                ChatMessage { role, content: Value::String(turn.text.clone()) }
            }
        }));
        text_answer_of(self.send(request, CHAT_TEMPERATURE, &messages))
    }
}

/// A user message; the document's page images go with it when the document was read as images.
fn user(text: &str, request: &AssistantRequest) -> ChatMessage {
    ChatMessage::user_with_images(text, images_of(request))
}

fn images_of(request: &AssistantRequest) -> &[PageImage] {
    match &request.content {
        DocumentContent::PageImages(pages) => pages,
        _ => &[],
    }
}
