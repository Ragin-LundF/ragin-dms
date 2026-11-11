use dms_application::common::OperationOutcome;
use dms_application::document::DetailEdit;
use dms_domain::catalog::CatalogRejection;
use dms_domain::field::{FieldPresets, FieldValue};
use dms_domain::retention::Retention;
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::Category;
use indexmap::IndexMap;

use crate::support::TestLibrary;
use crate::support::test_data::{category_id, date};

#[test]
fn notes_fields_retention_and_links_are_saved_to_every_folder() {
    let library = TestLibrary::new();
    library.open();
    let _ = library
        .change_settings()
        .run("fields", |_| CatalogSettings { field_definitions: FieldPresets::all(), ..CatalogSettings::default() });
    let bill = library.add("bill.pdf", "a");
    let reminder = library.add("reminder.pdf", "b");
    let due = FieldValue::Date(date(2024, 4, 1));
    let due_key = FieldPresets::due_date().key;

    let edits = [
        DetailEdit::SetNote { document_id: bill.id.clone(), note: Some("Paid twice?\nAsk the supplier".to_owned()) },
        DetailEdit::SetFields {
            document_id: bill.id.clone(),
            fields: IndexMap::from([(due_key.clone(), due.clone())]),
        },
        DetailEdit::SetRetention {
            document_id: bill.id.clone(),
            retention: Some(Retention { keep_until: Some(date(2030, 1, 1)), settled_at: None }),
        },
        DetailEdit::Link { first: reminder.id.clone(), second: bill.id.clone(), label: Some("reminder".to_owned()) },
    ];
    for edit in &edits {
        let outcome = library.edit_document_details().run(edit);
        assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    }

    for folder in library.all_folders() {
        let saved = library.folders.catalog_in(&folder).expect("a catalog");
        let document = &saved.documents()[&bill.id];
        assert_eq!(document.note.as_deref(), Some("Paid twice?\nAsk the supplier"));
        assert_eq!(document.fields.get(&due_key), Some(&due));
        assert_eq!(document.retention.as_ref().and_then(|retention| retention.keep_until), Some(date(2030, 1, 1)));
        let links = saved.links_of(&bill.id);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].label(), Some("reminder"));
    }
}

#[test]
fn a_field_the_library_does_not_define_is_refused() {
    let library = TestLibrary::new();
    library.open();
    let bill = library.add("bill.pdf", "a");

    let outcome = library.edit_document_details().run(&DetailEdit::SetFields {
        document_id: bill.id,
        fields: IndexMap::from([(FieldPresets::amount().key, FieldValue::Flag(true))]),
    });

    assert_eq!(outcome, OperationOutcome::Rejected(CatalogRejection::FieldNotDefined));
}

#[test]
fn a_trashed_document_loses_its_links() {
    let library = TestLibrary::new();
    library.open();
    let bill = library.add("bill.pdf", "a");
    let reminder = library.add("reminder.pdf", "b");
    library.edit_document_details().run(&DetailEdit::Link {
        first: bill.id.clone(),
        second: reminder.id.clone(),
        label: None,
    });

    library.trash_document().run(&reminder.id, true);

    assert!(library.catalog().links().is_empty());
}

#[test]
fn a_category_retention_period_is_opt_in_per_category() {
    let library = TestLibrary::new();
    library.open();
    let tax = Category::new(category_id("tax"), "Tax", None, None).expect("valid category");
    library.store.commit(|catalog| catalog.add_category(tax.clone()));

    library
        .edit_document_details()
        .run(&DetailEdit::SetRetentionYears { category_id: tax.id.clone(), years: Some(10) });

    assert_eq!(library.catalog().categories()[&tax.id].retention_years, Some(10));
}
