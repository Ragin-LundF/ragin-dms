use dms_application::contract::{ContractDraft, ExtractContractTermsOutcome};
use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{ContractTermsAnswer, ContractTermsReading, ReadPrice};
use dms_domain::contract::{ContractId, Notice, NoticeUnit, PricePoint};
use dms_domain::money::BillingPeriod;

use super::kit::{ContractKit, draft, euros};
use crate::support::fake::assistant::FakeContractTermsReader;
use crate::support::test_data::date;

fn read_price(valid_from: chrono::NaiveDate, amount: &str, source: Option<usize>) -> ReadPrice {
    ReadPrice { valid_from, price: euros(amount, BillingPeriod::Monthly), source }
}

#[test]
fn while_ai_is_off_nothing_is_asked() {
    let kit = ContractKit::new();
    let id = kit.record(&draft("Internet"));

    assert_eq!(kit.extract_terms().run(&id), ExtractContractTermsOutcome::AiDisabled);
    assert_eq!(kit.terms_reader.requests.get(), Vec::new());
}

#[test]
fn two_yearly_letters_become_price_points_with_their_source_documents_minus_the_known_ones() {
    let kit = ContractKit::new();
    kit.enable_ai();
    let newer = kit.add_dated("price-2024.pdf", date(2023, 11, 20), "From 1 January 2024: 44.99 EUR per month");
    let older = kit.add_dated("price-2023.pdf", date(2022, 11, 20), "From 1 January 2023: 39.99 EUR per month");
    let id = kit.record(&ContractDraft {
        price: Some(euros("39.99", BillingPeriod::Monthly)),
        price_valid_from: Some(date(2023, 1, 1)),
        document_ids: vec![newer.id.clone(), older.id.clone()],
        ..draft("Internet")
    });
    let notice = Notice { count: 1, unit: NoticeUnit::Months };
    kit.terms_reader.answer.set(ContractTermsAnswer::Read {
        reading: ContractTermsReading {
            prices: vec![
                read_price(date(2024, 1, 1), "44.99", Some(2)),
                read_price(date(2023, 1, 1), "39.99", Some(1)),
            ],
            notice: Some(notice),
            term_end: Some(date(2024, 12, 31)),
            renews_by_months: Some(12),
        },
        model: FakeContractTermsReader::MODEL.to_owned(),
    });

    let outcome = kit.extract_terms().run(&id);

    let ExtractContractTermsOutcome::Suggested(suggestion) = outcome else {
        panic!("expected a suggestion: {outcome:?}")
    };
    assert_eq!(
        suggestion.prices,
        vec![PricePoint {
            valid_from: date(2024, 1, 1),
            price: euros("44.99", BillingPeriod::Monthly),
            document_id: Some(newer.id),
        }]
    );
    assert_eq!(suggestion.notice, Some(notice));
    assert_eq!(suggestion.term_end, Some(date(2024, 12, 31)));
    assert_eq!(suggestion.renews_by_months, Some(12));
    let request = kit.terms_reader.requests.get().pop().expect("asked once");
    assert_eq!(request.contract_name, "Internet");
    assert_eq!(request.correspondent.as_deref(), Some("Net Provider"));
    assert!(request.source_texts[0].starts_with("price-2023.pdf (2022-11-20):"), "{:?}", request.source_texts);
    assert!(request.source_texts[1].starts_with("price-2024.pdf (2023-11-20):"), "{:?}", request.source_texts);
}

#[test]
fn a_source_number_out_of_range_leaves_the_point_without_a_document() {
    let kit = ContractKit::new();
    kit.enable_ai();
    let letter = kit.add_dated("price.pdf", date(2023, 11, 20), "44.99 EUR");
    let id = kit.record(&ContractDraft { document_ids: vec![letter.id], ..draft("Internet") });
    kit.terms_reader.answer.set(ContractTermsAnswer::Read {
        reading: ContractTermsReading {
            prices: vec![read_price(date(2024, 1, 1), "44.99", Some(7)), read_price(date(2024, 2, 1), "1.00", Some(0))],
            ..ContractTermsReading::default()
        },
        model: FakeContractTermsReader::MODEL.to_owned(),
    });

    let ExtractContractTermsOutcome::Suggested(suggestion) = kit.extract_terms().run(&id) else { panic!("suggestion") };

    assert!(suggestion.prices.iter().all(|point| point.document_id.is_none()));
}

#[test]
fn without_readable_text_unknown_contracts_and_failures_say_so() {
    let kit = ContractKit::new();
    kit.enable_ai();
    let silent = kit.add("silent.pdf", "silent");
    let id = kit.record(&ContractDraft { document_ids: vec![silent.id], ..draft("Internet") });

    assert_eq!(kit.extract_terms().run(&id), ExtractContractTermsOutcome::NoText);
    assert_eq!(kit.extract_terms().run(&ContractId::new("gone").expect("id")), ExtractContractTermsOutcome::NotFound);

    let letter = kit.add_dated("price.pdf", date(2023, 11, 20), "44.99 EUR");
    let with_text = kit.record(&ContractDraft { document_ids: vec![letter.id], ..draft("Phone") });
    kit.terms_reader
        .answer
        .set(ContractTermsAnswer::Failed { reason: ClassificationFailure::ProviderUnavailable, detail: None });
    assert_eq!(
        kit.extract_terms().run(&with_text),
        ExtractContractTermsOutcome::Failed(ClassificationFailure::ProviderUnavailable)
    );
}
