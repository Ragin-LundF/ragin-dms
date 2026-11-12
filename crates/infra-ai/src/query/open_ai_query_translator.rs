use std::sync::Arc;

use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{FieldAnswer, QueryAnswer, QueryRequest, QueryTranslator, TranslatedQuery};
use dms_domain::query::FieldComparison;
use dms_domain::tax::TaxArea;
use dms_domain::{parse_iso_date, text};

use super::query_answer_dto::{FieldConditionDto, QueryAnswerDto, query_schema};
use super::query_prompts::{self, payment_of, tax_area_key};
use crate::classify::suggestion_dto::outermost_object;
use crate::client::chat_request::{self, ChatMessage};
use crate::client::{ChatOptions, ChatReply, ChatTransport};

const TEMPERATURE: f64 = 0.0;
const SCHEMA_NAME: &str = "query";
const NO_QUERY: &str = "The answer is no filter";

/// Turns a sentence into filters on the chat model, constrained by the query schema. The sentence is not logged.
pub struct OpenAiQueryTranslator {
    transport: Arc<ChatTransport>,
}

impl OpenAiQueryTranslator {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }
}

impl QueryTranslator for OpenAiQueryTranslator {
    fn translate(&self, request: &QueryRequest) -> QueryAnswer {
        let messages =
            [ChatMessage::system(&query_prompts::system(request)), ChatMessage::user(&query_prompts::request(request))];
        let options = ChatOptions { structured: true, ..ChatOptions::default() };
        let reply = self.transport.complete(options, |model| {
            chat_request::structured(model, TEMPERATURE, &messages, SCHEMA_NAME, &query_schema())
        });
        match reply {
            ChatReply::Failed { reason, detail } => QueryAnswer::Failed { reason, detail },
            ChatReply::Answered { content, model } => match parse(&content) {
                Some(query) => QueryAnswer::Translated { query, model },
                None => QueryAnswer::Failed {
                    reason: ClassificationFailure::InvalidResponse,
                    detail: Some(NO_QUERY.to_owned()),
                },
            },
        }
    }
}

/// The outermost JSON object as filters; values that are no date, comparison, payment state or tax area are left
/// out, and so is a field condition with an unknown comparison. `None` for no JSON or another shape.
fn parse(content: &str) -> Option<TranslatedQuery> {
    let answer: QueryAnswerDto = serde_json::from_str(outermost_object(content)?).ok()?;
    Some(TranslatedQuery {
        text: cleaned(answer.text.as_deref()),
        category: cleaned(answer.category.as_deref()),
        tags: answer.tags.iter().filter_map(|tag| cleaned(Some(tag))).collect(),
        correspondent: cleaned(answer.correspondent.as_deref()),
        date_from: answer.date_from.as_deref().and_then(|date| parse_iso_date(text::trim(date))),
        date_to: answer.date_to.as_deref().and_then(|date| parse_iso_date(text::trim(date))),
        fields: answer.fields.iter().filter_map(field_of).collect(),
        payment: answer.payment.as_deref().and_then(payment_of),
        tax_year: answer.tax_year,
        tax_area: answer
            .tax_area
            .as_deref()
            .and_then(|key| TaxArea::ALL.into_iter().find(|area| tax_area_key(*area) == text::trim(key))),
    })
}

fn field_of(field: &FieldConditionDto) -> Option<FieldAnswer> {
    let comparison = match text::trim(&field.comparison) {
        "equals" => FieldComparison::Equals,
        "contains" => FieldComparison::Contains,
        "lessThan" => FieldComparison::LessThan,
        "greaterThan" => FieldComparison::GreaterThan,
        "present" => FieldComparison::Present,
        "absent" => FieldComparison::Absent,
        _ => return None,
    };
    let key = cleaned(Some(&field.key))?;
    let value = if matches!(comparison, FieldComparison::Present | FieldComparison::Absent) {
        None
    } else {
        Some(cleaned(field.value.as_deref())?)
    };
    Some(FieldAnswer { key, comparison, value })
}

/// Trimmed; empty and the word "null" count as missing.
fn cleaned(value: Option<&str>) -> Option<String> {
    let value = text::trim(value?);
    (!value.is_empty() && !value.eq_ignore_ascii_case("null")).then(|| value.to_owned())
}
