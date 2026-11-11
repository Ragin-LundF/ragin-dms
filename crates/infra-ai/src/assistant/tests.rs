#![allow(clippy::expect_used)]

use std::time::Duration;

use dms_application::port::ai::{ClassificationFailure, DocumentContent};
use dms_application::port::assistant::{
    AssistantAnswer, AssistantRequest, ChatRole, ChatTurn, DocumentAssistant, LibraryAssistant, SourceChunk,
};
use dms_application::port::state::AiSettings;
use dms_domain::taxonomy::TagName;
use indexmap::IndexSet;
use serde_json::Value;

use super::tag_answer_parser;
use crate::test_support::{Reply, StubServer, adapters_with, chat_answer, ok, tag, tiny_image, unreachable_base_url};

fn text_request() -> AssistantRequest {
    AssistantRequest {
        file_name: "scan-001.pdf".to_owned(),
        content: DocumentContent::Text("Sample Energy, bill 4711".to_owned()),
    }
}

fn content_of(message: &Value) -> &str {
    message["content"].as_str().expect("text content")
}

fn reason_of(answer: AssistantAnswer) -> ClassificationFailure {
    match answer {
        AssistantAnswer::Failed { reason, .. } => reason,
        other => panic!("answered: {other:?}"),
    }
}

// --- document assistant ---

#[test]
fn a_summary_sends_the_summary_prompt_and_the_document_text() {
    let server = StubServer::start(|_| ok(chat_answer("<think>hmm</think>A sample bill.")));
    let ai = server.adapters(AiSettings { model: Some("sample-model".to_owned()), ..AiSettings::default() }, None);

    let answer = ai.assistant.summarize(&text_request());

    assert_eq!(answer, AssistantAnswer::Text { text: "A sample bill.".to_owned(), model: "sample-model".to_owned() });
    let body = &server.chat_bodies()[0];
    assert_eq!(body["temperature"], 0.2);
    assert!(content_of(&body["messages"][0]).contains("summarize"));
    let user = content_of(&body["messages"][1]);
    assert!(user.contains("Read this document and summarize it."));
    assert!(user.contains("Sample Energy, bill 4711"));
}

#[test]
fn tags_are_read_from_the_answer_and_the_existing_tax_tag_is_named_in_the_prompt() {
    let server = StubServer::start(|_| ok(chat_answer("Tags: Invoice, tax, Electricity")));
    let ai = server.adapters(AiSettings { model: Some("sample-model".to_owned()), ..AiSettings::default() }, None);

    let answer = ai.assistant.suggest_tags(&text_request(), &IndexSet::from([tag("Tax")]));

    let AssistantAnswer::Tags { tags, .. } = answer else { panic!("no tags: {answer:?}") };
    assert_eq!(tags.iter().map(TagName::value).collect::<Vec<_>>(), ["Invoice", "Tax", "Electricity"]);
    assert!(content_of(&server.chat_bodies()[0]["messages"][0]).contains("include the tag \"Tax\""));
}

#[test]
fn an_answer_without_tags_is_invalid() {
    let server = StubServer::start(|_| ok(chat_answer("I cannot read this document, it seems to be empty.")));
    let ai = server.adapters(AiSettings { model: Some("m".to_owned()), ..AiSettings::default() }, None);

    let answer = ai.assistant.suggest_tags(&text_request(), &IndexSet::new());

    assert_eq!(reason_of(answer), ClassificationFailure::InvalidResponse);
}

#[test]
fn in_a_conversation_the_page_images_go_with_the_first_question_only() {
    let server = StubServer::start(|_| ok(chat_answer("On 15 March 2024.")));
    let ai = server.adapters(AiSettings { model: Some("m".to_owned()), ..AiSettings::default() }, None);
    let request = AssistantRequest {
        file_name: "scan-001.png".to_owned(),
        content: DocumentContent::PageImages(vec![tiny_image()]),
    };
    let conversation = [
        ChatTurn { role: ChatRole::User, text: "Who sent it?".to_owned() },
        ChatTurn { role: ChatRole::Assistant, text: "Sample Energy.".to_owned() },
        ChatTurn { role: ChatRole::User, text: "When?".to_owned() },
    ];

    ai.assistant.answer(&request, &conversation);

    let body = &server.chat_bodies()[0];
    let messages = body["messages"].as_array().expect("messages");
    let roles: Vec<_> = messages.iter().map(|message| message["role"].as_str().expect("role")).collect();
    assert_eq!(roles, ["system", "user", "assistant", "user"]);
    assert!(messages[1]["content"].is_array());
    assert!(messages[3]["content"].is_string());
    assert!(content_of(&messages[0]).contains("page images"));
    assert_eq!(body["temperature"], 0.3);
}

// --- library assistant ---

fn sources() -> Vec<SourceChunk> {
    vec![SourceChunk { number: 1, title: "Sample bill".to_owned(), text: "Meter: 120 kWh".to_owned() }]
}

fn chat_settings() -> AiSettings {
    AiSettings { model: Some("sample-chat".to_owned()), ..AiSettings::default() }
}

#[test]
fn the_answer_comes_without_thoughts_and_emphasis_with_the_model_that_gave_it() {
    let server = StubServer::start(|_| ok(chat_answer("<think>look</think>It was **120 kWh** [1].")));

    let answer = server.adapters(chat_settings(), None).library_assistant.answer("How much power?", &sources());

    assert_eq!(
        answer,
        AssistantAnswer::Text { text: "It was 120 kWh [1].".to_owned(), model: "sample-chat".to_owned() }
    );
}

#[test]
fn every_passage_is_sent_with_its_number_cut_to_its_maximum_length() {
    let server = StubServer::start(|_| ok(chat_answer("Done [2].")));
    let mut sources = sources();
    sources.push(SourceChunk { number: 2, title: "Sample contract".to_owned(), text: "y".repeat(5_000) });

    server.adapters(chat_settings(), None).library_assistant.answer("What does it say?", &sources);

    let body = &server.chat_bodies()[0];
    let system = content_of(&body["messages"][0]);
    assert!(system.contains(&format!("[1] Meter: 120 kWh\n\n[2] {}", "y".repeat(4_000))));
    assert!(!system.contains(&"y".repeat(4_001)));
    assert_eq!(content_of(&body["messages"][1]), "What does it say?");
}

#[test]
fn an_empty_answer_one_with_thoughts_only_or_one_without_choices_is_invalid() {
    let answers = [
        chat_answer(""),
        chat_answer("<think>The passages say nothing.</think>"),
        r#"{"choices":[]}"#.to_owned(),
        r#"{"choices":[{"message":{"role":"assistant"}}]}"#.to_owned(),
    ];
    for text in answers {
        let body = text.clone();
        let server = StubServer::start(move |_| ok(body.clone()));

        let answer = server.adapters(chat_settings(), None).library_assistant.answer("How much power?", &sources());

        assert_eq!(reason_of(answer), ClassificationFailure::InvalidResponse, "{text}");
    }
}

#[test]
fn an_answer_that_is_no_json_or_not_the_chat_schema_is_invalid() {
    let answers = [
        "<html>Internal error</html>",
        "",
        r#"{"choices":{"message":"It was 120 kWh."}}"#,
        r#"{"choices":[{"message":{"role":"assistant","content":["It was 120 kWh."]}}]}"#,
    ];
    for text in answers {
        let server = StubServer::start(move |_| ok(text));

        let answer = server.adapters(chat_settings(), None).library_assistant.answer("How much power?", &sources());

        assert_eq!(reason_of(answer), ClassificationFailure::InvalidResponse, "{text}");
    }
}

#[test]
fn a_refused_request_is_an_unavailable_provider_never_an_image_problem() {
    let server = StubServer::start(|_| Reply::Status(400, "context too long".to_owned()));

    let answer = server.adapters(chat_settings(), None).library_assistant.answer("How much power?", &sources());

    assert_eq!(
        answer,
        AssistantAnswer::Failed {
            reason: ClassificationFailure::ProviderUnavailable,
            detail: Some("HTTP 400: context too long".to_owned()),
        }
    );
}

#[test]
fn an_unreachable_server_and_a_server_that_takes_too_long_are_told_apart() {
    let unreachable = AiSettings { base_url: unreachable_base_url(), ..chat_settings() };
    let slow = StubServer::start(|_| Reply::Late(Duration::from_secs(3), chat_answer("late")));

    let refused = adapters_with(unreachable, None, &unreachable_base_url()).library_assistant.answer("?", &sources());
    let timed_out = slow
        .adapters(AiSettings { timeout_seconds: 1, ..chat_settings() }, None)
        .library_assistant
        .answer("?", &sources());

    assert_eq!(reason_of(refused), ClassificationFailure::ProviderUnavailable);
    assert_eq!(reason_of(timed_out), ClassificationFailure::Timeout);
}

#[test]
fn without_a_chat_model_on_the_server_the_question_is_not_sent() {
    let server = StubServer::start(|_| ok(r#"{"data":[{"id":"text-embedding-sample"}]}"#));

    let answer = server.adapters(AiSettings::default(), None).library_assistant.answer("How much power?", &sources());

    assert_eq!(
        answer,
        AssistantAnswer::Failed {
            reason: ClassificationFailure::ProviderUnavailable,
            detail: Some("The AI server has no chat model".to_owned()),
        }
    );
    assert_eq!(server.chat_bodies(), Vec::<Value>::new());
}

// --- tag answers ---

fn existing() -> IndexSet<TagName> {
    IndexSet::from([tag("Steuer"), tag("Strom")])
}

fn values(tags: &[TagName]) -> Vec<&str> {
    tags.iter().map(TagName::value).collect()
}

#[test]
fn a_plain_comma_separated_answer_is_taken_as_it_is() {
    let tags = tag_answer_parser::parse("Rechnung, Strom, Steuer", &existing());

    assert_eq!(values(&tags), ["Rechnung", "Strom", "Steuer"]);
}

#[test]
fn lead_ins_bullets_numbering_quotes_and_fences_are_removed() {
    let answer = "Here are the tags:\n```\n1. \"rechnung\"\n- #strom\n* steuer.\n```";

    let tags = tag_answer_parser::parse(answer, &existing());

    assert_eq!(values(&tags), ["rechnung", "Strom", "Steuer"]);
}

#[test]
fn sentences_are_no_tags_and_duplicates_are_dropped() {
    let answer = "Tags: Rechnung; rechnung, This document is clearly an electricity bill";

    let tags = tag_answer_parser::parse(answer, &existing());

    assert_eq!(values(&tags), ["Rechnung"]);
}

#[test]
fn windows_line_breaks_and_a_number_without_a_space_are_handled() {
    let tags = tag_answer_parser::parse("2024\r\n2) Strom\r\n3)x", &existing());

    assert_eq!(values(&tags), ["2024", "Strom", "3)x"]);
}
