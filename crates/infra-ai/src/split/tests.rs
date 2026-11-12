#![allow(clippy::expect_used)]

use chrono::NaiveDate;
use dms_application::port::ai::{ClassificationFailure, LineOutcome};
use dms_application::port::assistant::{SplitAnswer, SplitPage, SplitProposer, SplitRequest};
use dms_application::port::state::AiSettings;
use dms_domain::classification::SuggestedCategory;
use dms_domain::split::SplitPart;

use crate::test_support::{
    Reply, StubServer, adapters_with, chat_answer, energy, invoices, local_settings, ok, tiny_image,
    unreachable_base_url,
};

fn request() -> SplitRequest {
    SplitRequest {
        pages: vec![
            SplitPage { text: "Stadtwerke Beispielstadt Rechnung Seite 1 von 2".to_owned(), blank: false },
            SplitPage { text: "Seite 2 von 2 Gesamtbetrag".to_owned(), blank: false },
            SplitPage { text: String::new(), blank: true },
            SplitPage { text: "Finanzamt Beispielstadt Bescheid".to_owned(), blank: false },
        ],
        categories: vec![invoices(), energy()],
    }
}

fn propose_with(content: &str) -> (SplitAnswer, StubServer) {
    let answer = chat_answer(content);
    let server = StubServer::start(move |_| ok(answer.clone()));
    let result = server.adapters(local_settings(), None).split_proposer.propose(&request());
    (result, server)
}

fn reason_of(answer: SplitAnswer) -> ClassificationFailure {
    match answer {
        SplitAnswer::Failed { reason, .. } => reason,
        SplitAnswer::Proposed { parts, .. } => panic!("proposed: {parts:?}"),
    }
}

#[test]
fn the_parts_count_pages_from_zero_map_categories_and_dates_and_the_request_numbers_every_page() {
    let (answer, server) = propose_with(
        r#"{"parts":[
            {"firstPage":1,"lastPage":3,"title":" Stromrechnung März ","category":"invoices / energy",
             "correspondent":"Stadtwerke Beispielstadt","documentDate":"2026-03-02","confidence":0.92},
            {"firstPage":4,"lastPage":4,"title":"Steuerbescheid","category":"Tax","correspondent":null,
             "documentDate":"02.03.2026","confidence":7}
        ]}"#,
    );

    let expected = vec![
        SplitPart {
            title: Some("Stromrechnung März".to_owned()),
            category: Some(SuggestedCategory::Existing(energy().id)),
            correspondent: Some("Stadtwerke Beispielstadt".to_owned()),
            document_date: NaiveDate::from_ymd_opt(2026, 3, 2),
            confidence: Some(0.92),
            ..SplitPart::pages(0, 2)
        },
        SplitPart {
            title: Some("Steuerbescheid".to_owned()),
            category: Some(SuggestedCategory::New { name: "Tax".to_owned(), parent_id: None }),
            ..SplitPart::pages(3, 3)
        },
    ];
    assert_eq!(answer, SplitAnswer::Proposed { parts: expected, model: "sample-model".to_owned() });
    let body = &server.chat_bodies()[0];
    assert_eq!(body["response_format"]["json_schema"]["name"], "split");
    let user = body["messages"][1]["content"].as_str().expect("request text");
    assert!(user.contains("--- Page 1 ---\nStadtwerke Beispielstadt Rechnung"), "{user}");
    assert!(user.contains("--- Page 3 [blank] ---"), "{user}");
    assert!(user.contains("Existing categories: Invoices; Invoices / Energy"), "{user}");
}

#[test]
fn malformed_off_schema_or_page_zero_answers_are_an_invalid_response() {
    for content in [
        "The stack holds two documents.",
        r#"{"parts":[{"firstPage":1,"#,
        r#"{"documents":[{"from":1,"to":2}]}"#,
        r#"{"parts":[{"firstPage":0,"lastPage":1}]}"#,
        r#"{"parts":[{"firstPage":"one","lastPage":2}]}"#,
    ] {
        let (answer, _server) = propose_with(content);

        assert_eq!(reason_of(answer), ClassificationFailure::InvalidResponse, "{content}");
    }
}

#[test]
fn a_page_is_read_as_plain_text_with_its_image_and_without_a_schema() {
    let server = StubServer::start(|_| ok(chat_answer("  Rechnung Nr. 4711\nBetrag 12,30 €  ")));

    let outcome = server.adapters(local_settings(), None).split_proposer.read_page(&tiny_image());

    assert_eq!(outcome, LineOutcome::Read("Rechnung Nr. 4711\nBetrag 12,30 €".to_owned()));
    let body = &server.chat_bodies()[0];
    assert!(body.get("response_format").is_none());
    assert_eq!(body["messages"][0]["content"][1]["type"], "image_url");
}

#[test]
fn a_model_without_vision_and_an_unreachable_server_are_typed_failures() {
    let server = StubServer::start(|_| Reply::Status(400, "no vision".to_owned()));
    let refused = server.adapters(local_settings(), None).split_proposer.read_page(&tiny_image());
    let settings = AiSettings { base_url: unreachable_base_url(), ..local_settings() };
    let offline = adapters_with(settings, None, &unreachable_base_url()).split_proposer.propose(&request());

    assert!(
        matches!(refused, LineOutcome::Failure { reason: ClassificationFailure::ModelCannotReadImages, .. }),
        "{refused:?}"
    );
    assert_eq!(reason_of(offline), ClassificationFailure::ProviderUnavailable);
}
