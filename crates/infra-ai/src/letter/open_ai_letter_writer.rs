use std::sync::Arc;

use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{LetterAnswer, LetterDraft, LetterRequest, LetterWriter};
use dms_domain::text;

use super::letter_answer_dto::{LetterAnswerDto, letter_schema};
use super::letter_prompts;
use crate::classify::suggestion_dto::outermost_object;
use crate::client::chat_request::{self, ChatMessage};
use crate::client::{ChatOptions, ChatReply, ChatTransport};

const TEMPERATURE: f64 = 0.3;
const SCHEMA_NAME: &str = "letter";
const NO_LETTER: &str = "The answer is no letter";

/// Drafts letters on the chat model, constrained by the letter schema. Neither the documents nor the draft are
/// logged.
pub struct OpenAiLetterWriter {
    transport: Arc<ChatTransport>,
}

impl OpenAiLetterWriter {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }
}

impl LetterWriter for OpenAiLetterWriter {
    fn draft(&self, request: &LetterRequest) -> LetterAnswer {
        let messages = [
            ChatMessage::system(&letter_prompts::system(request)),
            ChatMessage::user(&letter_prompts::request(request)),
        ];
        let options = ChatOptions { structured: true, ..ChatOptions::default() };
        let reply = self.transport.complete(options, |model| {
            chat_request::structured(model, TEMPERATURE, &messages, SCHEMA_NAME, &letter_schema())
        });
        match reply {
            ChatReply::Failed { reason, detail } => LetterAnswer::Failed { reason, detail },
            ChatReply::Answered { content, model } => match parse(&content) {
                Some(draft) => LetterAnswer::Drafted { draft, model },
                None => LetterAnswer::Failed {
                    reason: ClassificationFailure::InvalidResponse,
                    detail: Some(NO_LETTER.to_owned()),
                },
            },
        }
    }
}

/// The outermost JSON object as a letter with a subject and a body that are not blank; `None` otherwise.
fn parse(content: &str) -> Option<LetterDraft> {
    let answer: LetterAnswerDto = serde_json::from_str(outermost_object(content)?).ok()?;
    let subject = text::trim(&answer.subject);
    let body = text::trim(&answer.body);
    (!subject.is_empty() && !body.is_empty())
        .then(|| LetterDraft { subject: subject.to_owned(), body: body.to_owned() })
}
