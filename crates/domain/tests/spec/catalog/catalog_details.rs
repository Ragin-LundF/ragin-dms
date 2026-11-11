use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::document::Document;
use dms_domain::field::{CurrencyCode, Decimal, FieldKey, FieldPresets, FieldValue};
use dms_domain::retention::Retention;
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::Category;
use indexmap::{IndexMap, indexmap};

use crate::test_data::{self, Applied, date, path};

struct Fixture {
    invoice: Document,
    reminder: Document,
    amount: FieldValue,
    with_presets: Catalog,
}

fn fixture() -> Fixture {
    let invoice = test_data::document("invoice", "letters/invoice.pdf");
    let reminder = test_data::document("reminder", "letters/reminder.pdf");
    let with_presets = test_data::catalog_with(&[invoice.clone(), reminder.clone()])
        .change_settings(CatalogSettings { field_definitions: FieldPresets::all(), ..CatalogSettings::default() })
        .applied();
    Fixture { invoice, reminder, amount: money("120.50"), with_presets }
}

fn money(amount: &str) -> FieldValue {
    FieldValue::Money {
        amount: Decimal::parse(amount).expect("valid amount"),
        currency: CurrencyCode::new("EUR").expect("valid code"),
    }
}

#[test]
fn field_values_need_a_definition() {
    let f = fixture();

    let update = test_data::catalog_with(std::slice::from_ref(&f.invoice))
        .set_fields(&f.invoice.id, indexmap! { FieldKey::new("amount").expect("valid key") => f.amount });

    assert_eq!(update, Ok(CatalogUpdate::Rejected(CatalogRejection::FieldNotDefined)));
}

#[test]
fn a_value_whose_definition_was_removed_may_stay_but_not_change() {
    let f = fixture();
    let amount_key = FieldPresets::amount().key;
    let with_amount =
        f.with_presets.set_fields(&f.invoice.id, indexmap! { amount_key.clone() => f.amount.clone() }).applied();
    let removed = with_amount.change_settings(CatalogSettings::default()).applied();

    let kept = removed.set_fields(&f.invoice.id, indexmap! { amount_key.clone() => f.amount });
    let changed = removed.set_fields(&f.invoice.id, indexmap! { amount_key => money("1") });

    assert!(matches!(kept, Ok(CatalogUpdate::Applied(_))));
    assert_eq!(changed, Ok(CatalogUpdate::Rejected(CatalogRejection::FieldNotDefined)));
}

#[test]
fn field_values_must_have_the_type_of_their_definition() {
    let f = fixture();

    let update = f
        .with_presets
        .set_fields(&f.invoice.id, indexmap! { FieldPresets::amount().key => FieldValue::Date(date(2026, 1, 1)) });

    assert_eq!(update, Ok(CatalogUpdate::Rejected(CatalogRejection::FieldTypeMismatch)));
}

#[test]
fn field_values_of_the_right_type_are_stored_with_the_next_revision() {
    let f = fixture();
    let fields: IndexMap<_, _> = indexmap! { FieldPresets::amount().key => f.amount };

    let catalog = f.with_presets.set_fields(&f.invoice.id, fields.clone()).applied();

    assert_eq!(catalog.documents()[&f.invoice.id].fields, fields);
    assert_eq!(catalog.revision(), f.with_presets.revision().next());
}

#[test]
fn a_blank_note_removes_the_note() {
    let f = fixture();

    let noted = f.with_presets.set_note(&f.invoice.id, Some("Paid by transfer\nsecond line")).applied();
    let cleared = noted.set_note(&f.invoice.id, Some("   ")).applied();

    assert_eq!(noted.documents()[&f.invoice.id].note.as_deref(), Some("Paid by transfer\nsecond line"));
    assert_eq!(cleared.documents()[&f.invoice.id].note, None);
}

#[test]
fn an_empty_retention_is_stored_as_none() {
    let f = fixture();

    let kept = f
        .with_presets
        .set_retention(&f.invoice.id, Some(Retention { keep_until: Some(date(2030, 12, 31)), settled_at: None }))
        .applied();
    let cleared = kept.set_retention(&f.invoice.id, Some(Retention::default())).applied();

    assert_eq!(
        kept.documents()[&f.invoice.id].retention.as_ref().and_then(|it| it.keep_until),
        Some(date(2030, 12, 31))
    );
    assert_eq!(cleared.documents()[&f.invoice.id].retention, None);
}

#[test]
fn many_documents_move_in_one_revision_and_may_swap_paths() {
    let f = fixture();

    let moved = f
        .with_presets
        .move_documents(&indexmap! { f.invoice.id.clone() => f.reminder.path.clone(), f.reminder.id.clone() => f.invoice.path.clone() })
        .applied();

    assert_eq!(moved.documents()[&f.invoice.id].path, f.reminder.path);
    assert_eq!(moved.documents()[&f.reminder.id].path, f.invoice.path);
    assert_eq!(moved.revision(), f.with_presets.revision().next());
}

#[test]
fn a_batch_move_that_puts_two_documents_on_one_path_is_refused() {
    let f = fixture();

    let update = f.with_presets.move_documents(&indexmap! { f.invoice.id.clone() => f.reminder.path.clone() });

    assert_eq!(update, Ok(CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed)));
}

#[test]
fn a_batch_move_of_an_unknown_document_is_refused() {
    let f = fixture();
    let ghost = test_data::document("ghost", "x.pdf");

    let update = f.with_presets.move_documents(&indexmap! { ghost.id => path("y.pdf") });

    assert!(matches!(update, Ok(CatalogUpdate::Rejected(_))));
}

#[test]
fn retention_years_of_a_category_are_opt_in_and_checked() {
    let f = fixture();
    let category = Category::new(test_data::category_id("tax"), "Tax", None, None).expect("valid category");

    let catalog =
        f.with_presets.add_category(category.clone()).applied().set_retention_years(&category.id, Some(10)).applied();

    assert_eq!(catalog.categories()[&category.id].retention_years, Some(10));
    assert_eq!(category.retention_years, None);
    assert!(Category { retention_years: Some(0), ..category }.validate().is_err());
}
