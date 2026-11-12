#![allow(clippy::expect_used)]

use chrono::NaiveDate;
use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{FieldAnswer, QueryAnswer, QueryRequest, QueryTranslator, TranslatedQuery};
use dms_application::port::state::AiSettings;
use dms_domain::bill::BillStatus;
use dms_domain::field::FieldPresets;
use dms_domain::query::FieldComparison;
use dms_domain::tax::TaxArea;

use crate::test_support::{
    StubServer, adapters_with, chat_answer, energy, invoices, local_settings, ok, tag, unreachable_base_url,
};

fn request() -> QueryRequest {
    QueryRequest {
        sentence: "Stadtwerke bills 2025 over 50 euros".to_owned(),
        today: NaiveDate::from_ymd_opt(2026, 11, 11).expect("date"),
        categories: vec![invoices(), energy()],
        tags: vec![tag("Tax")],
        correspondents: vec!["Stadtwerke Beispielstadt".to_owned()],
        fields: vec![FieldPresets::amount()],
        tax_areas: TaxArea::ALL.to_vec(),
    }
}

fn translate_with(content: &str) -> (QueryAnswer, StubServer) {
    let answer = chat_answer(content);
    let server = StubServer::start(move |_| ok(answer.clone()));
    let result = server.adapters(local_settings(), None).query_translator.translate(&request());
    (result, server)
}

fn translated(answer: QueryAnswer) -> TranslatedQuery {
    match answer {
        QueryAnswer::Translated { query, .. } => query,
        QueryAnswer::Failed { reason, detail } => panic!("failed: {reason:?} {detail:?}"),
    }
}

#[test]
fn a_full_answer_becomes_filters_and_the_request_lists_what_the_library_knows() {
    let (answer, server) = translate_with(
        r#"{"text":null,"category":"Invoices","tags":["Tax"],"correspondent":"Stadtwerke Beispielstadt",
            "dateFrom":"2025-01-01","dateTo":"2025-12-31",
            "fields":[{"key":"amount","comparison":"greaterThan","value":"50"},
                      {"key":"dueDate","comparison":"present","value":"ignored"}],
            "payment":"unpaid","taxYear":2025,"taxArea":"householdServices"}"#,
    );

    let query = translated(answer);
    assert_eq!(
        query,
        TranslatedQuery {
            text: None,
            category: Some("Invoices".to_owned()),
            tags: vec!["Tax".to_owned()],
            correspondent: Some("Stadtwerke Beispielstadt".to_owned()),
            date_from: NaiveDate::from_ymd_opt(2025, 1, 1),
            date_to: NaiveDate::from_ymd_opt(2025, 12, 31),
            fields: vec![
                FieldAnswer {
                    key: "amount".to_owned(),
                    comparison: FieldComparison::GreaterThan,
                    value: Some("50".to_owned())
                },
                FieldAnswer { key: "dueDate".to_owned(), comparison: FieldComparison::Present, value: None },
            ],
            payment: Some(BillStatus::Open),
            tax_year: Some(2025),
            tax_area: Some(TaxArea::HouseholdServices),
        }
    );
    let body = &server.chat_bodies()[0];
    assert_eq!(body["response_format"]["json_schema"]["name"], "query");
    let system = body["messages"][0]["content"].as_str().expect("system text");
    let user = body["messages"][1]["content"].as_str().expect("request text");
    assert!(system.contains("Today is 2026-11-11"), "{system}");
    assert!(user.contains("Existing categories: Invoices; Invoices / Energy"), "{user}");
    assert!(user.contains("Known correspondents: Stadtwerke Beispielstadt"), "{user}");
    assert!(user.contains("amount: Amount, MONEY") && user.contains("householdServices: "), "{user}");
    assert!(user.ends_with("Request: Stadtwerke bills 2025 over 50 euros"), "{user}");
}

#[test]
fn values_that_are_no_date_comparison_payment_or_tax_area_are_left_out() {
    let (answer, _server) = translate_with(
        r#"{"text":"  ","category":null,"tags":[" "],"correspondent":"null","dateFrom":"last year","dateTo":null,
            "fields":[{"key":"amount","comparison":"around","value":"50"},{"key":"amount","comparison":"lessThan","value":null}],
            "payment":"soon","taxYear":null,"taxArea":"holidays"}"#,
    );

    assert_eq!(translated(answer), TranslatedQuery::default());
}

#[test]
fn malformed_or_off_schema_answers_and_an_unreachable_server_are_typed_failures() {
    for content in ["Here are your filters.", r#"{"category": "#, r#"{"tags":"Tax"}"#] {
        let (answer, _server) = translate_with(content);

        assert!(
            matches!(answer, QueryAnswer::Failed { reason: ClassificationFailure::InvalidResponse, .. }),
            "{content}"
        );
    }
    let settings = AiSettings { base_url: unreachable_base_url(), ..local_settings() };
    let offline = adapters_with(settings, None, &unreachable_base_url()).query_translator.translate(&request());
    assert!(matches!(offline, QueryAnswer::Failed { reason: ClassificationFailure::ProviderUnavailable, .. }));
}
