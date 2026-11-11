use std::sync::Arc;

use dms_application::port::ai::{ClassificationFailure, LayoutOutcome, LineOutcome, PageImage, PageLayoutReader};

use super::layout_dto::LayoutAnswerDto;
use super::layout_image::layout_png;
use super::layout_mapper::to_layout;
use super::layout_schema::layout_schema;
use crate::classify::suggestion_dto::outermost_object;
use crate::client::chat_request::{self, ChatMessage};
use crate::client::{ChatOptions, ChatReply, ChatTransport};
use crate::prompt;

/// The pages took 1–4 minutes each with a 27B model, so the configured timeout is often too short.
pub(crate) const LAYOUT_TIMEOUT_SECONDS: i32 = 600;
const TEMPERATURE: f64 = 0.0;
const SCHEMA_NAME: &str = "page_layout";
const FENCE: &str = "```";
const NOT_AN_IMAGE: &str = "Not an image";
const NO_LAYOUT: &str = "The answer is no page layout";
const NO_TEXT: &str = "The AI answered nothing";

/// Reads page layouts with the configured vision model. The page goes out as PNG on the patch grid of Qwen-VL models,
/// the answer is constrained by the layout schema. A line is read again without a schema: some models write NUL for
/// every umlaut when the answer is forced into a JSON grammar. The document text is never logged.
///
/// A layout request waits up to [`LAYOUT_TIMEOUT_SECONDS`] or the configured timeout, whichever is longer; there is
/// no other way to stop it, so a timeout fails only that page, which then stays as it was.
pub struct AiPageLayoutReader {
    transport: Arc<ChatTransport>,
}

impl AiPageLayoutReader {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }
}

impl PageLayoutReader for AiPageLayoutReader {
    fn read_layout(&self, page: &PageImage) -> LayoutOutcome {
        let Some((sent, width, height)) = layout_png(page) else {
            return failure(ClassificationFailure::InvalidResponse, NOT_AN_IMAGE);
        };
        let request =
            prompt::REBUILD_LAYOUT_REQUEST.fill(&[("width", &width.to_string()), ("height", &height.to_string())]);
        let messages = [
            ChatMessage::system(&prompt::REBUILD_LAYOUT.text()),
            ChatMessage::user_with_images(&request, std::slice::from_ref(&sent)),
        ];
        let options = ChatOptions { has_images: true, structured: true, min_timeout_seconds: LAYOUT_TIMEOUT_SECONDS };
        let reply = self.transport.complete(options, |model| {
            chat_request::structured(model, TEMPERATURE, &messages, SCHEMA_NAME, &layout_schema())
        });
        match reply {
            ChatReply::Failed { reason, detail } => LayoutOutcome::Failure { reason, detail },
            ChatReply::Answered { content, .. } => match parse(&content) {
                Some(answer) => LayoutOutcome::Read(to_layout(answer, width, height)),
                None => failure(ClassificationFailure::InvalidResponse, NO_LAYOUT),
            },
        }
    }

    fn read_line(&self, line: &PageImage) -> LineOutcome {
        let messages = [ChatMessage::user_with_images(&prompt::REBUILD_LINE.text(), std::slice::from_ref(line))];
        let options = ChatOptions { has_images: true, ..ChatOptions::default() };
        let reply = self.transport.complete(options, |model| chat_request::chat(model, TEMPERATURE, &messages));
        match reply {
            ChatReply::Failed { reason, detail } => LineOutcome::Failure { reason, detail },
            ChatReply::Answered { content, .. } => match line_of(&content) {
                Some(text) => LineOutcome::Read(text.to_owned()),
                None => LineOutcome::Failure {
                    reason: ClassificationFailure::InvalidResponse,
                    detail: Some(NO_TEXT.to_owned()),
                },
            },
        }
    }
}

/// The outermost JSON object; a code fence or a sentence around it is dropped. `None` if it is no layout.
fn parse(content: &str) -> Option<LayoutAnswerDto> {
    serde_json::from_str(outermost_object(content)?).ok()
}

/// The last line of text without a code fence: models sometimes explain first.
fn line_of(content: &str) -> Option<&str> {
    content.lines().map(str::trim).filter(|line| !line.starts_with(FENCE)).rfind(|line| !line.is_empty())
}

fn failure(reason: ClassificationFailure, detail: &str) -> LayoutOutcome {
    LayoutOutcome::Failure { reason, detail: Some(detail.to_owned()) }
}
