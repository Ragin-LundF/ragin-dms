use chrono::{Days, NaiveDate};
use dms_application::classification::SuggestionApproval;
use dms_application::common::OperationOutcome;
use dms_application::port::ai::ClassificationResult;
use dms_application::port::system::Clock;
use dms_domain::bill::BillStatus;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{ClassificationSource, Document};
use dms_domain::field::{CurrencyCode, Decimal, FieldKey, FieldPresets, FieldValue};
use dms_domain::settings::CatalogSettings;
use dms_domain::task::OpenTask;
use indexmap::IndexMap;

use crate::support::TestLibrary;
use crate::support::fake::ai::FakeClassifier;

fn euros(amount: &str) -> FieldValue {
    FieldValue::Money {
        amount: Decimal::parse(amount).expect("decimal"),
        currency: CurrencyCode::new("EUR").expect("currency"),
    }
}

fn bill_fields(due: NaiveDate) -> IndexMap<FieldKey, FieldValue> {
    IndexMap::from([
        (FieldPresets::amount().key, euros("86.40")),
        (FieldPresets::due_date().key, FieldValue::Date(due)),
        (FieldPresets::iban().key, FieldValue::Text("DE89370400440532013000".to_owned())),
    ])
}

/// A library that knows no fields yet, and a document the AI suggests bill values for, approved unchanged.
fn approved_bill(library: &TestLibrary, due: NaiveDate) -> Document {
    library.open();
    let document = library.add("inbox/bill.pdf", "bill");
    library.classifier.result.set(Some(ClassificationResult::Suggested(ClassificationSuggestion {
        fields: bill_fields(due),
        ..ClassificationSuggestion::empty(document.id.clone(), FakeClassifier::MODEL)
    })));
    let _ = library.suggest_classification().run(&document.id);
    let approval = SuggestionApproval { fields: bill_fields(due), ..SuggestionApproval::default() };
    let outcome = library.approve_suggestion().run(&document.id, &approval);
    assert!(matches!(outcome, OperationOutcome::Done { .. }), "got {outcome:?}");
    library.catalog().documents()[&document.id].clone()
}

fn bill_tasks(library: &TestLibrary) -> Vec<OpenTask> {
    library
        .dashboard(None)
        .expect("a dashboard")
        .tasks
        .into_iter()
        .filter(|task| matches!(task, OpenTask::DeadlineDue { .. } | OpenTask::BillOverdue { .. }))
        .collect()
}

#[test]
fn ai_suggested_bill_values_survive_the_review_and_define_the_standard_fields_they_need() {
    let library = TestLibrary::new();
    let due = library.clock.today() + Days::new(3);

    let bill = approved_bill(&library, due);

    let catalog = library.catalog();
    assert_eq!(bill.fields, bill_fields(due));
    for preset in [FieldPresets::amount(), FieldPresets::due_date(), FieldPresets::iban()] {
        assert_eq!(catalog.settings().field(&preset.key), Some(&preset));
    }
    assert_eq!(BillStatus::of(&bill, catalog.settings(), library.clock.today()), Some(BillStatus::Open));
    assert_eq!(bill.classification.map(|record| record.source), Some(ClassificationSource::Ai));
}

#[test]
fn marking_paid_removes_the_due_date_task_and_undo_brings_it_back() {
    let library = TestLibrary::new();
    let bill = approved_bill(&library, library.clock.today() + Days::new(3));
    let before = bill_tasks(&library);

    let paid = library.mark_bill_paid().run(&bill.id, library.clock.today());
    let while_paid = bill_tasks(&library);
    let undone = library.mark_bill_unpaid().run(&bill.id);

    assert!(matches!(before.as_slice(), [OpenTask::DeadlineDue { .. }]), "got {before:?}");
    assert!(matches!(paid, OperationOutcome::Done { .. }), "got {paid:?}");
    assert_eq!(while_paid, Vec::new());
    assert!(matches!(undone, OperationOutcome::Done { .. }), "got {undone:?}");
    assert_eq!(bill_tasks(&library), before);
}

#[test]
fn an_overdue_bill_is_asked_about_once_and_paying_it_ends_that() {
    let library = TestLibrary::new();
    let due = library.clock.today() - Days::new(2);
    let bill = approved_bill(&library, due);

    let overdue = bill_tasks(&library);
    let _ = library.mark_bill_paid().run(&bill.id, library.clock.today());

    assert_eq!(overdue, vec![OpenTask::BillOverdue { document_id: bill.id.clone(), due }]);
    assert_eq!(bill_tasks(&library), Vec::new());
}

#[test]
fn older_bills_are_settled_on_their_due_date_only_after_confirmation() {
    let library = TestLibrary::new();
    let old_due = library.clock.today() - Days::new(60);
    let old = approved_bill(&library, old_due);
    let since = library.clock.today() - Days::new(30);
    library
        .change_settings()
        .run("tracking", |settings| CatalogSettings { bills_tracked_since: Some(since), ..settings.clone() });

    let unconfirmed = library.settle_older_bills().run(false);
    let settled = library.settle_older_bills().run(true);

    assert_eq!(unconfirmed, OperationOutcome::NotConfirmed);
    assert!(matches!(settled, OperationOutcome::Done { .. }), "got {settled:?}");
    assert_eq!(
        library.catalog().documents()[&old.id].fields.get(&FieldPresets::paid_on().key),
        Some(&FieldValue::Date(old_due))
    );
}
