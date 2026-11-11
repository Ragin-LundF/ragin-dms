use std::sync::Arc;

use dms_application::port::ai::{
    AiServer, ClassificationFailure, ClassificationRequest, ClassificationResult, DocumentClassifier, DocumentContent,
    ModelList,
};
use dms_application::port::state::AiSettings;
use serde_json::Value;

use super::suggestion_dto::SuggestionDto;
use super::suggestion_schema::suggestion_schema;
use super::{prompt_texts, suggestion_mapper};
use crate::client::chat_request::{self, ChatMessage};
use crate::client::{ChatOptions, ChatReply, ChatTransport, shortened};

const CLASSIFICATION_TEMPERATURE: f64 = 0.1;
const SCHEMA_NAME: &str = "document_classification";

/// Classifies documents and lists the models of any server with the OpenAI chat completions API, or of Anthropic.
pub struct OpenAiCompatibleClient {
    transport: Arc<ChatTransport>,
}

impl OpenAiCompatibleClient {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }
}

impl AiServer for OpenAiCompatibleClient {
    fn models(&self, settings: &AiSettings) -> ModelList {
        self.transport.models(settings)
    }
}

impl DocumentClassifier for OpenAiCompatibleClient {
    fn suggest(&self, request: &ClassificationRequest) -> ClassificationResult {
        let options = ChatOptions {
            has_images: matches!(request.content, DocumentContent::PageImages(_)),
            structured: true,
            ..ChatOptions::default()
        };
        match self.transport.complete(options, |model| body_of(model, request)) {
            ChatReply::Failed { reason, detail } => ClassificationResult::Failed { reason, detail },
            ChatReply::Answered { content, model } => match SuggestionDto::parse(&content) {
                Some(answer) => {
                    ClassificationResult::Suggested(suggestion_mapper::to_suggestion(&answer, request, &model))
                }
                None => ClassificationResult::Failed {
                    reason: ClassificationFailure::InvalidResponse,
                    detail: Some(shortened(&content)),
                },
            },
        }
    }
}

/// The rules and taxonomy, the document, and the JSON schema of the answer.
pub(crate) fn body_of(model: &str, request: &ClassificationRequest) -> Value {
    let (text, images) = match &request.content {
        DocumentContent::Text(text) => (Some(text.as_str()), &[][..]),
        DocumentContent::NotReadable(_) => (Some(""), &[][..]),
        DocumentContent::PageImages(pages) => (None, pages.as_slice()),
    };
    let messages = [
        ChatMessage::system(&prompt_texts::system(request)),
        ChatMessage::user_with_images(&prompt_texts::user(request, text), images),
    ];
    chat_request::structured(model, CLASSIFICATION_TEMPERATURE, &messages, SCHEMA_NAME, &suggestion_schema())
}
