use dms_application::catalog::CatalogEdit;
use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{FieldAnswer, QueryAnswer, TranslatedQuery};
use dms_application::search::{QueryTranslation, TranslateOutcome, TranslateQuery};
use dms_domain::bill::BillStatus;
use dms_domain::document::DocumentMetadata;
use dms_domain::field::{CurrencyCode, Decimal, FieldPresets, FieldValue};
use dms_domain::query::{DateRange, FieldComparison, FieldCondition};
use dms_domain::settings::CatalogSettings;
use dms_domain::tax::TaxArea;
use dms_domain::taxonomy::Category;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::fake::assistant::FakeQueryTranslator;
use crate::support::test_data::{category_id, date, done, tag};

struct Kit {
    library: TestLibrary,
    translator: std::sync::Arc<FakeQueryTranslator>,
}

impl Kit {
    /// A library with the categories "Home" › "Bills", the tag "Tax", the amount field (EUR by default) and a
    /// document from "Stadtwerke Beispielstadt".
    fn new() -> Self {
        let library = TestLibrary::new();
        library.open();
        let home = Category::new(category_id("home"), "Home", None, None).expect("valid category");
        let bills = Category::new(category_id("bills"), "Bills", Some(home.id.clone()), None).expect("valid category");
        done(library.store.commit(|catalog| catalog.add_category(home.clone())));
        done(library.store.commit(|catalog| catalog.add_category(bills.clone())));
        let mut settings =
            CatalogSettings { field_definitions: vec![FieldPresets::amount()], ..CatalogSettings::default() };
        settings.layout.default_currency = CurrencyCode::parse("EUR");
        done(library.store.commit(|catalog| catalog.change_settings(settings.clone())));
        let bill = library.add("bill.pdf", "water bill");
        for edit in [
            CatalogEdit::AddTags { document_id: bill.id.clone(), tags: IndexSet::from([tag("Tax")]) },
            CatalogEdit::UpdateMetadata {
                document_id: bill.id.clone(),
                metadata: DocumentMetadata {
                    correspondent: Some("Stadtwerke Beispielstadt".to_owned()),
                    ..DocumentMetadata::default()
                },
            },
        ] {
            let _ = library.edit_catalog().run(&edit);
        }
        Self { library, translator: std::sync::Arc::default() }
    }

    fn translate(&self, sentence: &str) -> TranslateOutcome {
        TranslateQuery::new(self.library.store.clone(), self.translator.clone(), self.library.clock.clone())
            .run(sentence)
    }

    fn translated(&self, answer: TranslatedQuery) -> QueryTranslation {
        self.translator.answer_with(answer);
        match self.translate("Stadtwerke bills 2025 over 50 euros") {
            TranslateOutcome::Translated(translation) => translation,
            other => panic!("expected a translation, got {other:?}"),
        }
    }
}

#[test]
fn known_names_become_the_query_and_the_request_carries_what_the_library_knows() {
    let kit = Kit::new();

    let translation = kit.translated(TranslatedQuery {
        category: Some("home / bills".to_owned()),
        tags: vec!["tax".to_owned()],
        correspondent: Some("stadtwerke beispielstadt".to_owned()),
        date_from: Some(date(2025, 1, 1)),
        date_to: Some(date(2025, 12, 31)),
        fields: vec![FieldAnswer {
            key: "Amount".to_owned(),
            comparison: FieldComparison::GreaterThan,
            value: Some("50".to_owned()),
        }],
        payment: Some(BillStatus::Open),
        tax_year: Some(2025),
        tax_area: Some(TaxArea::HouseholdServices),
        ..TranslatedQuery::default()
    });

    let query = &translation.query;
    assert_eq!(query.category, Some(category_id("bills")));
    assert_eq!(query.tags.all, IndexSet::from([tag("Tax")]));
    assert_eq!(query.correspondent.as_deref(), Some("Stadtwerke Beispielstadt"));
    assert_eq!(query.dates, Some(DateRange::year(2025).expect("valid year")));
    let fifty = FieldValue::Money {
        amount: Decimal::parse("50").expect("number"),
        currency: CurrencyCode::parse("EUR").expect("EUR"),
    };
    assert_eq!(
        query.fields,
        vec![
            FieldCondition::new(FieldPresets::amount().key, FieldComparison::GreaterThan, Some(fifty)).expect("valid")
        ]
    );
    assert_eq!(
        (translation.payment, translation.tax_year, translation.tax_area),
        (Some(BillStatus::Open), Some(2025), Some(TaxArea::HouseholdServices))
    );
    assert_eq!(translation.not_understood, Vec::<String>::new());
    let request = kit.translator.requests.get().pop().expect("one request");
    assert_eq!(request.sentence, "Stadtwerke bills 2025 over 50 euros");
    assert_eq!(request.correspondents, ["Stadtwerke Beispielstadt"]);
    assert_eq!(request.categories.len(), 2);
    assert_eq!(request.tax_areas.len(), TaxArea::ALL.len());
}

#[test]
fn an_unknown_sender_category_tag_or_field_is_reported_and_never_guessed() {
    let kit = Kit::new();

    let translation = kit.translated(TranslatedQuery {
        text: Some(" heating ".to_owned()),
        category: Some("Insurance".to_owned()),
        tags: vec!["Holiday".to_owned()],
        correspondent: Some("Stadtwerke".to_owned()),
        fields: vec![
            FieldAnswer { key: "iban".to_owned(), comparison: FieldComparison::Present, value: None },
            FieldAnswer {
                key: "amount".to_owned(),
                comparison: FieldComparison::GreaterThan,
                value: Some("lots".to_owned()),
            },
        ],
        date_from: Some(date(2026, 1, 1)),
        date_to: Some(date(2025, 1, 1)),
        ..TranslatedQuery::default()
    });

    assert_eq!(translation.not_understood, ["Insurance", "Holiday", "Stadtwerke", "iban", "amount", "dates"]);
    assert_eq!(translation.query.text.as_deref(), Some("heating"));
    let query = dms_domain::query::DocumentQuery { text: Some("heating".to_owned()), ..Default::default() };
    assert_eq!(translation.query, query);
}

#[test]
fn a_blank_sentence_asks_nothing_and_a_failure_is_passed_on() {
    let kit = Kit::new();
    kit.translator.answer.set(QueryAnswer::Failed { reason: ClassificationFailure::Timeout, detail: None });

    assert_eq!(kit.translate("   "), TranslateOutcome::BlankSentence);
    assert_eq!(kit.translator.requests.get().len(), 0);
    assert_eq!(
        kit.translate("unpaid bills"),
        TranslateOutcome::Failed { reason: ClassificationFailure::Timeout, detail: None }
    );
}
