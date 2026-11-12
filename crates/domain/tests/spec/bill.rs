use dms_domain::bill::{BillPlanner, BillStatus, EpcPayment, EpcRefusal, Iban};
use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::document::Document;
use dms_domain::field::{FieldKey, FieldPresets, FieldValue};
use dms_domain::settings::CatalogSettings;
use dms_domain::task::OpenTask;
use indexmap::IndexMap;

use crate::test_data::{self, Applied, date, decimal, document, document_id, euro, euros};

#[test]
fn an_iban_is_read_as_people_write_it_and_printed_in_groups() {
    let iban = Iban::parse(" de89 3704 0044 0532 0130 00 ").expect("valid IBAN");

    assert_eq!(iban.value(), "DE89370400440532013000");
    assert_eq!(iban.grouped(), "DE89 3704 0044 0532 0130 00");
    assert_eq!(Iban::parse("GB82 WEST 1234 5698 7654 32").expect("valid IBAN").value(), "GB82WEST12345698765432");
}

#[test]
fn an_iban_with_a_wrong_checksum_or_shape_is_refused() {
    for bad in ["DE89 3704 0044 0532 0130 01", "DE89", "1289 3704 0044 0532 0130 00", "DE8X370400440532013000", ""] {
        assert!(Iban::parse(bad).is_err(), "{bad}");
    }
}

fn settings() -> CatalogSettings {
    CatalogSettings {
        field_definitions: vec![FieldPresets::amount(), FieldPresets::due_date(), FieldPresets::paid_on()],
        ..CatalogSettings::default()
    }
}

fn bill(due: Option<(i32, u32, u32)>, paid: bool) -> Document {
    let mut bill = document("bill", "bill.pdf");
    bill.fields.insert(FieldPresets::amount().key, FieldValue::Money { amount: decimal("86.40"), currency: euro() });
    if let Some((year, month, day)) = due {
        bill.fields.insert(FieldPresets::due_date().key, FieldValue::Date(date(year, month, day)));
    }
    if paid {
        bill.fields.insert(FieldPresets::paid_on().key, FieldValue::Date(date(2026, 11, 2)));
    }
    bill
}

#[test]
fn a_bill_is_open_until_its_due_date_passed_and_paid_once_it_has_a_paid_on_date() {
    let today = date(2026, 11, 11);

    assert_eq!(BillStatus::of(&bill(Some((2026, 11, 11)), false), &settings(), today), Some(BillStatus::Open));
    assert_eq!(BillStatus::of(&bill(Some((2026, 11, 10)), false), &settings(), today), Some(BillStatus::Overdue));
    assert_eq!(BillStatus::of(&bill(Some((2026, 11, 1)), true), &settings(), today), Some(BillStatus::Paid));
}

#[test]
fn a_document_without_amount_or_due_date_is_no_bill() {
    let today = date(2026, 11, 11);
    let mut without_amount = bill(Some((2026, 11, 1)), false);
    without_amount.fields.shift_remove(&FieldPresets::amount().key);

    assert_eq!(BillStatus::of(&bill(None, false), &settings(), today), None);
    assert_eq!(BillStatus::of(&without_amount, &settings(), today), None);
    assert_eq!(
        BillStatus::of(&bill(Some((2026, 11, 1)), false), &CatalogSettings::default(), today),
        None,
        "no due date role defined"
    );
}

// --- EPC payment code

/// The sample of EPC069-12 (version 002), line by line.
const EPC069_12_SAMPLE: [&str; 12] = [
    "BCD",
    "002",
    "1",
    "SCT",
    "BPOTBEB1",
    "Red Cross of Belgium",
    "BE72000000001616",
    "EUR1",
    "CHAR",
    "",
    "Urgency fund",
    "Sample EPC QR code",
];

fn red_cross(amount: &str, reference: Option<&str>) -> Result<EpcPayment, EpcRefusal> {
    let iban = Iban::parse("BE72 0000 0000 1616").expect("valid IBAN");
    EpcPayment::new("Red Cross of Belgium", iban, Some("BPOTBEB1"), &euros(amount), reference)
}

#[test]
fn the_payload_matches_the_epc069_12_sample() {
    // The app sets no purpose code and no note to the payer, so those two lines of the sample stay empty.
    let mut expected = EPC069_12_SAMPLE[..11].to_vec();
    expected[8] = "";

    let payment = red_cross("1.00", Some("Urgency fund")).expect("a payment");

    assert_eq!(payment.payload(), expected.join("\n"));
}

#[test]
fn the_payload_leaves_out_empty_optional_lines_and_rounds_to_cents() {
    let iban = Iban::parse("DE89 3704 0044 0532 0130 00").expect("valid IBAN");
    let payment =
        EpcPayment::new(" Sample Energy ", iban, Some("not a bic"), &euros("86.405"), None).expect("a payment");

    assert_eq!(payment.payload(), "BCD\n002\n1\nSCT\n\nSample Energy\nDE89370400440532013000\nEUR86.41");
}

#[test]
fn a_payment_is_refused_outside_euro_amount_and_length_limits() {
    let iban = || Iban::parse("BE72 0000 0000 1616").expect("valid IBAN");
    let francs = dms_domain::money::Money {
        amount: decimal("10"),
        currency: dms_domain::field::CurrencyCode::new("CHF").expect("valid currency"),
    };

    assert!(matches!(EpcPayment::new("Payee", iban(), None, &francs, None), Err(EpcRefusal::NotEuro(_))));
    assert_eq!(red_cross("0.004", None), Err(EpcRefusal::AmountOutOfRange));
    assert_eq!(red_cross("1000000000", None), Err(EpcRefusal::AmountOutOfRange));
    assert!(red_cross("999999999.99", None).is_ok());
    assert_eq!(EpcPayment::new("  ", iban(), None, &euros("1"), None), Err(EpcRefusal::PayeeMissing));
    assert_eq!(EpcPayment::new(&"ä".repeat(71), iban(), None, &euros("1"), None), Err(EpcRefusal::PayeeTooLong));
    assert!(EpcPayment::new(&"ä".repeat(70), iban(), None, &euros("1"), None).is_ok());
    assert_eq!(red_cross("1", Some(&"x".repeat(141))), Err(EpcRefusal::ReferenceTooLong));
}

// --- Bill tasks and catalog changes

fn bills_catalog(bills: &[(&str, (i32, u32, u32), bool)]) -> Catalog {
    let documents: Vec<Document> = bills
        .iter()
        .map(|(id, due, paid)| {
            let mut document = bill(Some(*due), *paid);
            document.id = document_id(id);
            document.path = test_data::path(&format!("{id}.pdf"));
            document
        })
        .collect();
    test_data::catalog_with(&documents).change_settings(settings()).applied()
}

#[test]
fn every_unpaid_bill_past_its_due_date_within_the_lead_time_becomes_one_task_before_tracking_started() {
    let catalog = bills_catalog(&[
        ("late", (2026, 11, 10), false),
        ("old", (2026, 10, 1), false),
        ("paid", (2026, 11, 9), true),
        ("open", (2026, 11, 20), false),
    ]);

    let tasks = BillPlanner::due_tasks(&catalog, date(2026, 11, 11));

    assert_eq!(tasks, vec![OpenTask::BillOverdue { document_id: document_id("late"), due: date(2026, 11, 10) }]);
}

#[test]
fn once_tracking_started_bills_due_since_then_are_tasks_and_older_ones_are_not() {
    let catalog = bills_catalog(&[("late", (2026, 10, 20), false), ("older", (2026, 9, 30), false)])
        .change_settings(CatalogSettings { bills_tracked_since: Some(date(2026, 10, 1)), ..settings() })
        .applied();

    let tasks = BillPlanner::due_tasks(&catalog, date(2026, 11, 11));

    assert_eq!(tasks, vec![OpenTask::BillOverdue { document_id: document_id("late"), due: date(2026, 10, 20) }]);
}

#[test]
fn marking_paid_defines_the_paid_on_field_on_first_use_and_undo_clears_it() {
    let catalog = test_data::catalog_with(&[bill(Some((2026, 11, 1)), false)])
        .change_settings(CatalogSettings {
            field_definitions: vec![FieldPresets::amount(), FieldPresets::due_date()],
            ..CatalogSettings::default()
        })
        .applied();
    let id = document_id("bill");

    let paid = catalog.set_paid_on(&IndexMap::from([(id.clone(), Some(date(2026, 11, 5)))])).applied();
    let undone = paid.set_paid_on(&IndexMap::from([(id.clone(), None)])).applied();

    assert_eq!(paid.settings().field(&FieldPresets::paid_on().key), Some(&FieldPresets::paid_on()));
    assert_eq!(
        paid.documents()[&id].fields.get(&FieldPresets::paid_on().key),
        Some(&FieldValue::Date(date(2026, 11, 5)))
    );
    assert_eq!(
        BillStatus::of(&undone.documents()[&id], undone.settings(), date(2026, 11, 11)),
        Some(BillStatus::Overdue)
    );
    assert_eq!(
        catalog.set_paid_on(&IndexMap::from([(document_id("unknown"), None)])),
        CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound)
    );
}

#[test]
fn standard_fields_are_defined_only_where_missing() {
    let catalog = test_data::catalog_with(&[]).change_settings(settings()).applied();
    let keys = [FieldPresets::amount().key, FieldPresets::iban().key, FieldKey::new("custom").expect("valid key")];

    let defined = catalog.define_standard_fields(&keys).applied();

    let defined_keys: Vec<&str> = defined.settings().field_definitions.iter().map(|it| it.key.value()).collect();
    assert_eq!(defined_keys, vec!["amount", "due", "paid-on", "iban"]);
    assert_eq!(defined.define_standard_fields(&keys).applied().revision(), defined.revision(), "nothing missing");
}
