use dms_application::classification::SuggestOutcome;
use dms_application::port::ai::{ClassificationFailure, ClassificationResult};
use dms_application::port::state::TaskStore;
use dms_domain::bill::{EInvoice, Iban};
use dms_domain::classification::{ClassificationSuggestion, EInvoicePart};
use dms_domain::field::{CurrencyCode, Decimal, FieldPresets, FieldValue};
use dms_domain::money::Money;

use crate::support::TestLibrary;
use crate::support::fake::ai::FakeClassifier;
use crate::support::test_data::date;

fn e_invoice() -> EInvoice {
    EInvoice {
        seller: Some("Sample Energy GmbH".to_owned()),
        invoice_number: Some("RE-2024-0042".to_owned()),
        issue_date: Some(date(2024, 2, 20)),
        due_date: Some(date(2024, 3, 15)),
        amount: Some(Money {
            amount: Decimal::parse("86.40").expect("decimal"),
            currency: CurrencyCode::new("EUR").expect("currency"),
        }),
        iban: Some(Iban::parse("DE89370400440532013000").expect("iban")),
        bic: None,
        payment_reference: None,
    }
}

fn library_with_e_invoice() -> (TestLibrary, dms_domain::document::Document) {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/invoice.pdf", "invoice");
    library.reader.e_invoices.update(|all| all.insert("invoice.pdf".to_owned(), e_invoice()));
    (library, document)
}

fn assert_exact_values(suggestion: &ClassificationSuggestion) {
    assert_eq!(suggestion.correspondent.as_deref(), Some("Sample Energy GmbH"));
    assert_eq!(suggestion.document_date, Some(date(2024, 2, 20)));
    assert_eq!(suggestion.fields.get(&FieldPresets::due_date().key), Some(&FieldValue::Date(date(2024, 3, 15))));
    assert_eq!(
        suggestion.fields.get(&FieldPresets::payment_reference().key),
        Some(&FieldValue::Text("RE-2024-0042".to_owned())),
        "the invoice number when no payment reference is given"
    );
    assert_eq!(
        suggestion.e_invoice,
        vec![
            EInvoicePart::Correspondent,
            EInvoicePart::DocumentDate,
            EInvoicePart::Field(FieldPresets::amount().key),
            EInvoicePart::Field(FieldPresets::due_date().key),
            EInvoicePart::Field(FieldPresets::iban().key),
            EInvoicePart::Field(FieldPresets::payment_reference().key),
        ]
    );
}

#[test]
fn without_the_ai_an_e_invoice_still_makes_a_suggestion_with_its_exact_values() {
    let (library, document) = library_with_e_invoice();
    library
        .classifier
        .result
        .set(Some(ClassificationResult::Failed { reason: ClassificationFailure::ProviderUnavailable, detail: None }));

    let outcome = library.suggest_classification().run(&document.id);

    let SuggestOutcome::Suggested(suggestion) = outcome else { panic!("expected Suggested, got {outcome:?}") };
    assert_eq!(suggestion.model, ClassificationSuggestion::E_INVOICE_MODEL);
    assert!(!suggestion.by_ai());
    assert_exact_values(&suggestion);
    assert_eq!(library.tasks.suggestions().expect("readable"), [suggestion]);
}

#[test]
fn the_e_invoice_values_replace_what_the_ai_guessed_and_the_ai_keeps_the_rest() {
    let (library, document) = library_with_e_invoice();
    library.classifier.result.set(Some(ClassificationResult::Suggested(ClassificationSuggestion {
        title: Some("Electricity bill".to_owned()),
        correspondent: Some("Sample Enrgy".to_owned()),
        ..ClassificationSuggestion::empty(document.id.clone(), FakeClassifier::MODEL)
    })));

    let SuggestOutcome::Suggested(suggestion) = library.suggest_classification().run(&document.id) else {
        panic!("expected Suggested")
    };

    assert_eq!(suggestion.title.as_deref(), Some("Electricity bill"));
    assert_eq!(suggestion.model, "test-model + e-invoice");
    assert!(suggestion.by_ai());
    assert_exact_values(&suggestion);
}

#[test]
fn the_e_invoice_alone_is_suggested_on_request_and_a_pdf_without_one_says_so() {
    let (library, document) = library_with_e_invoice();
    let other = library.add("inbox/other.pdf", "other");

    let found = library.suggest_from_e_invoice().run(&document.id);
    let missing = library.suggest_from_e_invoice().run(&other.id);

    let SuggestOutcome::Suggested(suggestion) = found else { panic!("expected Suggested, got {found:?}") };
    assert_exact_values(&suggestion);
    assert_eq!(missing, SuggestOutcome::NotReadable { detail: None });
    assert_eq!(library.classifier.requests.get().len(), 0, "the AI is not asked");
}
