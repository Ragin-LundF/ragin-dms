#![allow(clippy::expect_used)]

use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{
    ContractTermsAnswer, ContractTermsReader, ContractTermsReading, ContractTermsRequest, ReadPrice,
};
use dms_application::port::state::AiSettings;
use dms_domain::contract::{ContractPrice, Notice, NoticeUnit};
use dms_domain::field::{CurrencyCode, Decimal};
use dms_domain::money::BillingPeriod;
use dms_domain::parse_iso_date;

use crate::test_support::{StubServer, adapters_with, chat_answer, local_settings, ok, unreachable_base_url};

fn request() -> ContractTermsRequest {
    ContractTermsRequest {
        contract_name: "Internet 250".to_owned(),
        correspondent: Some("Net Provider".to_owned()),
        source_texts: vec![
            "price-2023.pdf (2022-11-20):\nAb 1. Januar 2023 kostet Ihr Tarif 39,99 €".to_owned(),
            "price-2024.pdf (2023-11-20):\nAb 1. Januar 2024 kostet Ihr Tarif 44,99 €".to_owned(),
        ],
    }
}

fn read_with(content: &str) -> (ContractTermsAnswer, StubServer) {
    let answer = chat_answer(content);
    let server = StubServer::start(move |_| ok(answer.clone()));
    let result = server.adapters(local_settings(), None).contract_terms_reader.read(&request());
    (result, server)
}

fn monthly(amount: &str) -> ContractPrice {
    ContractPrice {
        amount: Decimal::parse(amount).expect("decimal"),
        currency: CurrencyCode::new("EUR").expect("currency"),
        period: BillingPeriod::Monthly,
    }
}

fn day(text: &str) -> chrono::NaiveDate {
    parse_iso_date(text).expect("date")
}

#[test]
fn two_yearly_letters_become_two_price_points_and_the_terms_with_numbered_documents_in_the_request() {
    let (answer, server) = read_with(
        r#"{"prices":[
            {"validFrom":"2023-01-01","amount":"39.99","currency":"EUR","period":"monthly","source":1},
            {"validFrom":"2024-01-01","amount":"44.99","currency":"eur","period":"monthly","source":2}],
           "noticeCount":1,"noticeUnit":"months","termEnd":"2024-12-31","renewsByMonths":12}"#,
    );

    assert_eq!(
        answer,
        ContractTermsAnswer::Read {
            reading: ContractTermsReading {
                prices: vec![
                    ReadPrice { valid_from: day("2023-01-01"), price: monthly("39.99"), source: Some(1) },
                    ReadPrice { valid_from: day("2024-01-01"), price: monthly("44.99"), source: Some(2) },
                ],
                notice: Some(Notice { count: 1, unit: NoticeUnit::Months }),
                term_end: Some(day("2024-12-31")),
                renews_by_months: Some(12),
            },
            model: "sample-model".to_owned(),
        }
    );
    let body = &server.chat_bodies()[0];
    assert_eq!(body["response_format"]["json_schema"]["name"], "contract_terms");
    let user = body["messages"][1]["content"].as_str().expect("request text");
    assert!(user.contains("Contract: Internet 250") && user.contains("Correspondent: Net Provider"), "{user}");
    assert!(user.contains("[1] price-2023.pdf") && user.contains("[2] price-2024.pdf"), "{user}");
}

#[test]
fn an_answer_without_prices_or_terms_is_an_empty_reading() {
    let (answer, _server) =
        read_with(r#"{"prices":[],"noticeCount":null,"noticeUnit":null,"termEnd":null,"renewsByMonths":null}"#);

    assert!(
        matches!(answer, ContractTermsAnswer::Read { ref reading, .. } if *reading == ContractTermsReading::default()),
        "{answer:?}"
    );
}

#[test]
fn malformed_or_off_schema_answers_are_an_invalid_response_never_a_partial_reading() {
    let price = |valid_from: &str, amount: &str, period: &str| {
        format!(
            r#"{{"prices":[{{"validFrom":"{valid_from}","amount":"{amount}","currency":"EUR","period":"{period}","source":null}}],"noticeCount":null,"noticeUnit":null,"termEnd":null,"renewsByMonths":null}}"#
        )
    };
    for content in [
        "I cannot read this contract.".to_owned(),
        r#"{"prices": ["#.to_owned(),
        price("1 January 2024", "44.99", "monthly"),
        price("2024-01-01", "44,99", "monthly"),
        price("2024-01-01", "-5.00", "monthly"),
        price("2024-01-01", "44.99", "weekly"),
        r#"{"prices":[],"noticeCount":3,"noticeUnit":null,"termEnd":null,"renewsByMonths":null}"#.to_owned(),
        r#"{"prices":[],"noticeCount":0,"noticeUnit":"months","termEnd":null,"renewsByMonths":null}"#.to_owned(),
        r#"{"prices":[],"noticeCount":null,"noticeUnit":null,"termEnd":"soon","renewsByMonths":null}"#.to_owned(),
        r#"{"prices":[],"noticeCount":null,"noticeUnit":null,"termEnd":null,"renewsByMonths":0}"#.to_owned(),
    ] {
        let (answer, _server) = read_with(&content);

        assert!(
            matches!(answer, ContractTermsAnswer::Failed { reason: ClassificationFailure::InvalidResponse, .. }),
            "{content}: {answer:?}"
        );
    }
}

#[test]
fn an_unreachable_server_is_a_typed_failure() {
    let settings = AiSettings { base_url: unreachable_base_url(), ..local_settings() };

    let answer = adapters_with(settings, None, &unreachable_base_url()).contract_terms_reader.read(&request());

    assert!(matches!(answer, ContractTermsAnswer::Failed { reason: ClassificationFailure::ProviderUnavailable, .. }));
}
