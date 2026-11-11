#![allow(clippy::expect_used)]

use std::io::Cursor;
use std::time::Duration;

use dms_application::port::ai::{ClassificationFailure, LayoutOutcome, LineOutcome, PageImage, PageLayoutReader};
use dms_application::port::state::AiSettings;
use dms_domain::rebuild::{FontFamily, LayoutBox, PageLayout};
use image::{DynamicImage, ImageFormat, RgbImage};
use serde_json::json;

use super::ai_page_layout_reader::LAYOUT_TIMEOUT_SECONDS;
use crate::client::request_timeout;
use crate::test_support::{Reply, StubServer, chat_answer, ok};

const EMPTY_LAYOUT: &str = r#"{"lines":[],"graphics":[]}"#;

fn settings() -> AiSettings {
    AiSettings { model: Some("sample-model".to_owned()), timeout_seconds: 60, ..AiSettings::default() }
}

/// A synthetic page of 2000 x 1000 pixels; it goes out as 1568 x 784.
fn page() -> PageImage {
    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(RgbImage::new(2000, 1000))
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .expect("png");
    PageImage { bytes, mime_type: "image/png".to_owned() }
}

fn layout_of(outcome: LayoutOutcome) -> PageLayout {
    match outcome {
        LayoutOutcome::Read(layout) => layout,
        LayoutOutcome::Failure { reason, detail } => panic!("failed: {reason:?} {detail:?}"),
    }
}

fn reason_of(outcome: LayoutOutcome) -> ClassificationFailure {
    match outcome {
        LayoutOutcome::Failure { reason, .. } => reason,
        LayoutOutcome::Read(layout) => panic!("read: {layout:?}"),
    }
}

fn read_with(answer: String) -> LayoutOutcome {
    let server = StubServer::start(move |_| ok(answer.clone()));
    server.adapters(settings(), None).layout_reader.read_layout(&page())
}

#[test]
fn an_answer_in_0_to_1000_keeps_its_boxes_and_its_umlauts() {
    let content = r#"{"lines":[{"bbox":[100,200,500,240],"text":"Prüfung für Müller","bold":true,"font":"serif","legible":true}],"graphics":[{"kind":"logo","bbox":[800,50,950,150]}]}"#;

    let layout = layout_of(read_with(chat_answer(content)));

    let line = &layout.lines[0];
    assert_eq!(line.bounds, LayoutBox { left: 100.0, top: 200.0, right: 500.0, bottom: 240.0 });
    assert_eq!(line.text, "Prüfung für Müller");
    assert_eq!(line.family, FontFamily::Serif);
    assert!(line.bold && line.legible);
    assert_eq!(layout.graphics[0].kind, "logo");
}

#[test]
fn an_answer_in_pixels_of_the_sent_image_is_converted_to_0_to_1000() {
    let content = r#"{"lines":[{"bbox":[784,392,1568,784],"text":"Beispiel","bold":false,"font":"sans","legible":true}],"graphics":[]}"#;

    let layout = layout_of(read_with(chat_answer(content)));

    assert_eq!(layout.lines[0].bounds, LayoutBox { left: 500.0, top: 500.0, right: 1000.0, bottom: 1000.0 });
}

#[test]
fn boxes_are_ordered_clipped_and_lines_without_four_values_are_dropped() {
    let content = r#"{"lines":[{"bbox":[500,240,100,-20],"text":"a","font":"MONO"},{"bbox":[1,2,3],"text":"b"}],"graphics":[{"kind":null,"bbox":[0,0,10,10]}]}"#;

    let layout = layout_of(read_with(chat_answer(content)));

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].bounds, LayoutBox { left: 100.0, top: 0.0, right: 500.0, bottom: 240.0 });
    assert_eq!(layout.lines[0].family, FontFamily::Mono);
    assert!(!layout.lines[0].legible, "not legible unless the AI says so");
    assert_eq!(layout.graphics[0].kind, "graphic");
}

#[test]
fn the_page_goes_out_as_png_on_the_patch_grid_with_the_schema() {
    let server = StubServer::start(|_| ok(chat_answer(EMPTY_LAYOUT)));

    server.adapters(settings(), None).layout_reader.read_layout(&page());

    let body = &server.chat_bodies()[0];
    assert_eq!(body["response_format"]["json_schema"]["name"], "page_layout");
    assert_eq!(body["temperature"], 0.0);
    let user = &body["messages"][1]["content"];
    assert!(user[0]["text"].as_str().expect("text").contains("1568 x 784"));
    assert!(user[1]["image_url"]["url"].as_str().expect("url").starts_with("data:image/png;base64,"));
}

#[test]
fn a_layout_request_waits_at_least_ten_minutes() {
    assert_eq!(request_timeout(&settings(), LAYOUT_TIMEOUT_SECONDS), Duration::from_secs(600));
}

#[test]
fn an_answer_only_in_the_reasoning_content_is_read() {
    let body = json!({"choices": [{"message": {"content": "", "reasoning_content": EMPTY_LAYOUT}}]});

    layout_of(read_with(body.to_string()));
}

#[test]
fn a_thinking_block_and_a_code_fence_around_the_answer_are_dropped() {
    layout_of(read_with(chat_answer(&format!("<think>looking at the page</think>\n```json\n{EMPTY_LAYOUT}\n```"))));
}

#[test]
fn http_400_on_an_image_means_the_model_cannot_read_images() {
    let server = StubServer::start(|_| Reply::Status(400, "no vision".to_owned()));

    let outcome = server.adapters(settings(), None).layout_reader.read_layout(&page());

    assert_eq!(reason_of(outcome), ClassificationFailure::ModelCannotReadImages);
}

#[test]
fn a_request_that_runs_out_of_time_is_a_timeout() {
    let server = StubServer::start(|_| Reply::Late(Duration::from_secs(3), chat_answer("late")));

    // Reading a line has no long minimum wait, so the configured second runs out.
    let outcome =
        server.adapters(AiSettings { timeout_seconds: 1, ..settings() }, None).layout_reader.read_line(&page());

    assert!(matches!(outcome, LineOutcome::Failure { reason: ClassificationFailure::Timeout, .. }), "{outcome:?}");
}

#[test]
fn broken_json_and_a_picture_that_is_no_image_are_invalid() {
    let server = StubServer::start(|_| ok(chat_answer(r#"{"lines":[{"bbox":[1,2"#)));
    let reader = server.adapters(settings(), None).layout_reader;

    let broken = reader.read_layout(&page());
    let not_an_image = reader.read_layout(&PageImage { bytes: vec![1, 2, 3], mime_type: "image/png".to_owned() });

    assert_eq!(reason_of(broken), ClassificationFailure::InvalidResponse);
    assert_eq!(
        not_an_image,
        LayoutOutcome::Failure {
            reason: ClassificationFailure::InvalidResponse,
            detail: Some("Not an image".to_owned())
        }
    );
    assert_eq!(server.received().len(), 1, "no request for a picture that is no image");
}

#[test]
fn a_line_is_read_again_as_plain_text_without_a_schema() {
    let server = StubServer::start(|_| ok(chat_answer("Here it is:\n```\nGrüße aus Köln\n```")));

    let outcome = server.adapters(settings(), None).layout_reader.read_line(&page());

    assert_eq!(outcome, LineOutcome::Read("Grüße aus Köln".to_owned()));
    let body = &server.chat_bodies()[0];
    assert!(body.get("response_format").is_none());
    assert_eq!(body["messages"].as_array().expect("messages").len(), 1);
}
