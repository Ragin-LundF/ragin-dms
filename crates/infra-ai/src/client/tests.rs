#![allow(clippy::expect_used)]

use dms_application::port::state::AiSettings;
use serde_json::json;

use super::anthropic_request::anthropic_body;
use super::chat_request::{self, ChatMessage};
use super::chat_transport::chat_model_of;
use super::*;
use crate::test_support::tiny_image;

#[test]
fn thoughts_and_bold_markers_are_removed() {
    let answer = "<think>Let me check the date.</think>\nBis **31.03.2024** zahlen.";

    assert_eq!(AnswerText::plain(answer), "Bis 31.03.2024 zahlen.");
}

#[test]
fn an_answer_after_a_lone_closing_think_tag_is_kept() {
    assert_eq!(AnswerText::clean("thoughts</think> Answer"), "Answer");
}

#[test]
fn several_think_blocks_are_removed_and_an_unclosed_one_stays() {
    assert_eq!(AnswerText::clean("<think>a</think>One <think>b\nc</think>two"), "One two");
    assert_eq!(AnswerText::clean("<think>still thinking"), "<think>still thinking");
}

#[test]
fn system_messages_move_to_system_images_become_base64_blocks_and_temperature_is_dropped() {
    let open_ai = chat_request::chat(
        "claude-opus-5-5",
        0.2,
        &[ChatMessage::system("Be brief."), ChatMessage::user_with_images("Read", &[tiny_image()])],
    );

    let body = anthropic_body(&open_ai);

    assert_eq!(body["system"], "Be brief.");
    assert!(body.get("temperature").is_none());
    assert_eq!(body["max_tokens"], 16_000);
    let messages = body["messages"].as_array().expect("messages");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["role"], "user");
    let source = &messages[0]["content"][1]["source"];
    assert_eq!(source["media_type"], "image/jpeg");
    assert_eq!(source["data"], "AQID");
    assert_eq!(messages[0]["content"][0], json!({"type": "text", "text": "Read"}));
}

#[test]
fn the_answer_schema_becomes_a_structured_output() {
    let open_ai = json!({
        "model": "claude-opus-5-5",
        "messages": [],
        "response_format": {"json_schema": {"schema": {"type": "object"}}},
    });

    let body = anthropic_body(&open_ai);

    assert_eq!(body["output_config"]["format"]["type"], "json_schema");
    assert_eq!(body["output_config"]["format"]["schema"]["type"], "object");
    assert!(body.get("system").is_none());
}

#[test]
fn embedding_models_are_skipped() {
    let models = ["text-embedding-sample-v1", "multilingual-e5-small", "sample-chat-model"].map(String::from);

    assert_eq!(chat_model_of(&models).as_deref(), Some("sample-chat-model"));
}

#[test]
fn no_chat_model_gives_none() {
    assert_eq!(chat_model_of(&["sample-embedding".to_owned()]), None);
}

#[test]
fn a_request_waits_the_configured_time_or_longer_for_slow_requests() {
    let settings = AiSettings { timeout_seconds: 60, ..AiSettings::default() };

    assert_eq!(request_timeout(&settings, 0).as_secs(), 60);
    assert_eq!(request_timeout(&settings, 600).as_secs(), 600);
    assert_eq!(request_timeout(&AiSettings { timeout_seconds: -5, ..settings }, 0).as_secs(), 1);
}

#[test]
fn the_base_url_is_trimmed_and_loses_its_trailing_slashes() {
    let settings = AiSettings { base_url: " http://localhost:1234/v1// ".to_owned(), ..AiSettings::default() };

    assert_eq!(local_base_of(&settings), "http://localhost:1234/v1");
}
