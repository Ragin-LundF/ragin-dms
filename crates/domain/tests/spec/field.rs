use dms_domain::field::{
    CurrencyCode, Decimal, FieldDefinition, FieldKey, FieldRole, FieldType, FieldValue, FieldValues,
};

use crate::test_data::date;

fn euro() -> CurrencyCode {
    CurrencyCode::new("EUR").expect("valid code")
}

fn money(amount: &str, currency: &str) -> FieldValue {
    FieldValue::Money { amount: decimal(amount), currency: CurrencyCode::new(currency).expect("valid code") }
}

fn decimal(text: &str) -> Decimal {
    Decimal::parse(text).expect("valid decimal")
}

#[test]
fn money_takes_the_currency_it_names_before_or_after_the_amount() {
    assert_eq!(FieldValues::parse(FieldType::Money, "CHF 1'234.50", Some(&euro())), Some(money("1234.50", "CHF")));
    assert_eq!(FieldValues::parse(FieldType::Money, "1,234.5 usd", Some(&euro())), Some(money("1234.5", "USD")));
}

#[test]
fn money_without_a_currency_takes_the_librarys_default_and_without_one_it_is_refused() {
    assert_eq!(FieldValues::parse(FieldType::Money, "99,90", Some(&euro())), Some(money("99.90", "EUR")));
    assert_eq!(FieldValues::parse(FieldType::Money, "99,90", None), None);
}

#[test]
fn values_that_do_not_fit_the_type_are_refused() {
    assert_eq!(FieldValues::parse(FieldType::Number, "twelve", None), None);
    assert_eq!(FieldValues::parse(FieldType::Date, "31.02.2026", None), None);
    assert_eq!(FieldValues::parse(FieldType::Boolean, "maybe", None), None);
    assert_eq!(FieldValues::parse(FieldType::Text, "   ", None), None);
}

#[test]
fn every_type_reads_back_what_format_writes() {
    let values = [
        FieldValue::text("Contract 7").expect("valid text"),
        FieldValue::Number(decimal("-3.25")),
        money("1000.00", "EUR"),
        FieldValue::Date(date(2026, 12, 1)),
        FieldValue::Flag(true),
    ];

    for value in values {
        let text = FieldValues::format(&value);
        assert_eq!(FieldValues::parse(value.field_type(), &text, None), Some(value));
    }
}

#[test]
fn keys_are_derived_from_labels() {
    assert_eq!(FieldKey::from_label("Contract end"), Some(FieldKey::new("contract-end").expect("valid")));
    assert_eq!(FieldKey::from_label(" Betrag! "), Some(FieldKey::new("betrag").expect("valid")));
    assert_eq!(FieldKey::from_label("123"), None);
}

#[test]
fn only_date_fields_can_be_deadlines() {
    let definition =
        FieldDefinition::new(FieldKey::new("x").expect("valid"), "X", FieldType::Text, Some(FieldRole::DueDate));

    assert!(definition.is_err());
}
