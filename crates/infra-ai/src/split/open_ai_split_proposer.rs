use std::sync::Arc;

use dms_application::port::ai::{ClassificationFailure, LineOutcome, PageImage};
use dms_application::port::assistant::{SplitAnswer, SplitProposer, SplitRequest};
use dms_domain::split::SplitPart;
use dms_domain::taxonomy::Category;
use dms_domain::{parse_iso_date, text};

use super::split_answer_dto::{SplitAnswerDto, SplitPartDto, split_schema};
use super::split_prompts;
use crate::classify::suggestion_dto::outermost_object;
use crate::classify::suggestion_mapper::category_of;
use crate::client::chat_request::{self, ChatMessage};
use crate::client::{ChatOptions, ChatReply, ChatTransport};
use crate::prompt;

const TEMPERATURE: f64 = 0.1;
const SCHEMA_NAME: &str = "split";
const NO_SPLIT: &str = "The answer is no split";

/// Reads scanned pages and proposes the parts of a stack on the chat model, constrained by the split schema. Neither
/// the pages nor the answer are logged.
pub struct OpenAiSplitProposer {
    transport: Arc<ChatTransport>,
}

impl OpenAiSplitProposer {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }
}

impl SplitProposer for OpenAiSplitProposer {
    fn read_page(&self, page: &PageImage) -> LineOutcome {
        let messages = [ChatMessage::user_with_images(&prompt::SPLIT_PAGE.text(), std::slice::from_ref(page))];
        let options = ChatOptions { has_images: true, ..ChatOptions::default() };
        match self.transport.complete(options, |model| chat_request::chat(model, TEMPERATURE, &messages)) {
            ChatReply::Failed { reason, detail } => LineOutcome::Failure { reason, detail },
            ChatReply::Answered { content, .. } => LineOutcome::Read(text::trim(&content).to_owned()),
        }
    }

    fn propose(&self, request: &SplitRequest) -> SplitAnswer {
        let messages =
            [ChatMessage::system(&prompt::SPLIT.text()), ChatMessage::user(&split_prompts::request(request))];
        let options = ChatOptions { structured: true, ..ChatOptions::default() };
        let reply = self.transport.complete(options, |model| {
            chat_request::structured(model, TEMPERATURE, &messages, SCHEMA_NAME, &split_schema())
        });
        match reply {
            ChatReply::Failed { reason, detail } => SplitAnswer::Failed { reason, detail },
            ChatReply::Answered { content, model } => match parse(&content, &request.categories) {
                Some(parts) => SplitAnswer::Proposed { parts, model },
                None => SplitAnswer::Failed {
                    reason: ClassificationFailure::InvalidResponse,
                    detail: Some(NO_SPLIT.to_owned()),
                },
            },
        }
    }
}

/// The parts of the outermost JSON object, pages counted from 0; `None` for no JSON, another shape or a page 0.
fn parse(content: &str, categories: &[Category]) -> Option<Vec<SplitPart>> {
    let answer: SplitAnswerDto = serde_json::from_str(outermost_object(content)?).ok()?;
    answer.parts.iter().map(|part| part_of(part, categories)).collect()
}

fn part_of(part: &SplitPartDto, categories: &[Category]) -> Option<SplitPart> {
    Some(SplitPart {
        first_page: part.first_page.checked_sub(1)?,
        last_page: part.last_page.checked_sub(1)?,
        title: cleaned(part.title.as_deref()),
        category: cleaned(part.category.as_deref()).and_then(|name| category_of(&name, categories)),
        correspondent: cleaned(part.correspondent.as_deref()),
        document_date: part.document_date.as_deref().and_then(|date| parse_iso_date(text::trim(date))),
        confidence: part.confidence.filter(|confidence| (0.0..=1.0).contains(confidence)),
    })
}

/// Trimmed; empty and the word "null" count as missing.
fn cleaned(value: Option<&str>) -> Option<String> {
    let value = text::trim(value?);
    (!value.is_empty() && !value.eq_ignore_ascii_case("null")).then(|| value.to_owned())
}
