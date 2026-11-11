use chrono_tz::Tz;
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::field::{CurrencyCode, Decimal, FieldKey, FieldPresets, FieldValue};
use dms_domain::query::{DateRange, DocumentQuery, FieldComparison, FieldCondition, QueryContext, TagFilter};
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::{Category, TagName};
use indexmap::{IndexSet, indexmap, indexset};

use crate::test_data::{self, Applied, category_id, date, instant, path, tag};

fn insurance() -> Category {
    Category::new(category_id("insurance"), "Insurance", None, None).expect("valid")
}

fn car() -> Category {
    Category::new(category_id("car"), "Car", Some(insurance().id), None).expect("valid")
}

fn tax() -> TagName {
    tag("Tax")
}

fn paid() -> TagName {
    tag("Paid")
}

fn money(amount: &str) -> FieldValue {
    FieldValue::Money { amount: number(amount), currency: CurrencyCode::new("EUR").expect("valid") }
}

fn number(text: &str) -> Decimal {
    Decimal::parse(text).expect("valid number")
}

fn catalog() -> Catalog {
    let policy = Document {
        category_id: Some(car().id),
        tags: indexset! { tax() },
        ..test_data::document("policy", "insurance/policy.pdf")
    };
    let invoice =
        Document { tags: indexset! { tax(), paid() }, ..test_data::document("invoice", "letters/invoice.pdf") };
    let photo = test_data::document("photo", "photo.jpg");
    test_data::catalog_with(&[])
        .add_category(insurance())
        .applied()
        .add_category(car())
        .applied()
        .change_settings(CatalogSettings { field_definitions: FieldPresets::all(), ..CatalogSettings::default() })
        .applied()
        .add_document(policy.clone())
        .applied()
        .add_document(invoice.clone())
        .applied()
        .add_document(photo.clone())
        .applied()
        .update_metadata(
            &invoice.id,
            DocumentMetadata::new(Some("Garage bill".to_owned()), Some(date(2025, 6, 1)), Some("Garage".to_owned()))
                .expect("valid"),
        )
        .applied()
        .set_note(&photo.id, Some("Scratch on the left door"))
        .applied()
        .set_fields(
            &invoice.id,
            indexmap! {
                FieldPresets::amount().key => money("250"),
                FieldPresets::due_date().key => FieldValue::Date(date(2026, 3, 20)),
            },
        )
        .applied()
        .set_fields(&policy.id, indexmap! { FieldPresets::amount().key => money("80") })
        .applied()
}

fn ids(query: &DocumentQuery) -> IndexSet<String> {
    let catalog = catalog();
    let context = QueryContext { catalog: &catalog, today: date(2026, 3, 1), zone: Tz::UTC };
    query.filter(&context).iter().map(|document| document.id.value().to_owned()).collect()
}

fn set(values: &[&str]) -> IndexSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn with_text(text: &str) -> DocumentQuery {
    DocumentQuery { text: Some(text.to_owned()), ..DocumentQuery::default() }
}

fn with_tags(tags: TagFilter) -> DocumentQuery {
    DocumentQuery { tags, ..DocumentQuery::default() }
}

fn with_field(condition: FieldCondition) -> DocumentQuery {
    DocumentQuery { fields: vec![condition], ..DocumentQuery::default() }
}

fn condition(key: FieldKey, comparison: FieldComparison, value: Option<FieldValue>) -> FieldCondition {
    FieldCondition::new(key, comparison, value).expect("valid condition")
}

#[test]
fn an_empty_query_matches_everything() {
    assert_eq!(ids(&DocumentQuery::default()), set(&["policy", "invoice", "photo"]));
}

#[test]
fn text_looks_at_path_title_correspondent_tags_and_note() {
    assert_eq!(ids(&with_text("garage BILL")), set(&["invoice"]));
    assert_eq!(ids(&with_text("left door")), set(&["photo"]));
    assert_eq!(ids(&with_text("insurance/")), set(&["policy"]));
    assert_eq!(ids(&with_text("paid")), set(&["invoice"]));
}

#[test]
fn a_category_includes_its_subcategories() {
    assert_eq!(ids(&DocumentQuery { category: Some(insurance().id), ..DocumentQuery::default() }), set(&["policy"]));
    assert_eq!(ids(&DocumentQuery { uncategorized: true, ..DocumentQuery::default() }), set(&["invoice", "photo"]));
}

#[test]
fn tags_can_be_required_all_any_or_none() {
    assert_eq!(
        ids(&with_tags(TagFilter { all: indexset! { tax(), paid() }, ..TagFilter::default() })),
        set(&["invoice"])
    );
    assert_eq!(
        ids(&with_tags(TagFilter { any: indexset! { tax() }, ..TagFilter::default() })),
        set(&["policy", "invoice"])
    );
    assert_eq!(
        ids(&with_tags(TagFilter { none: indexset! { paid() }, ..TagFilter::default() })),
        set(&["policy", "photo"])
    );
    assert_eq!(ids(&DocumentQuery { untagged: true, ..DocumentQuery::default() }), set(&["photo"]));
}

#[test]
fn dates_use_the_document_date_otherwise_the_day_it_was_added() {
    let in_2025 = DocumentQuery { dates: Some(DateRange::year(2025).expect("valid")), ..DocumentQuery::default() };
    let until_2024 =
        DocumentQuery { dates: Some(DateRange::until_end_of(2024).expect("valid")), ..DocumentQuery::default() };

    assert_eq!(ids(&in_2025), set(&["invoice"]));
    assert_eq!(ids(&until_2024), set(&["policy", "photo"]));
}

#[test]
fn correspondent_and_folder_narrow_the_result() {
    assert_eq!(
        ids(&DocumentQuery { correspondent: Some(" garage ".to_owned()), ..DocumentQuery::default() }),
        set(&["invoice"])
    );
    assert_eq!(ids(&DocumentQuery { folder: Some(path("insurance")), ..DocumentQuery::default() }), set(&["policy"]));
}

#[test]
fn field_conditions_compare_amounts_and_dates() {
    let amount = FieldPresets::amount().key;
    let due = FieldPresets::due_date().key;
    let expensive = condition(amount.clone(), FieldComparison::GreaterThan, Some(FieldValue::Number(number("100"))));
    let due_before = condition(due.clone(), FieldComparison::LessThan, Some(FieldValue::Date(date(2026, 12, 1))));
    let due_soon = condition(due, FieldComparison::WithinDays, Some(FieldValue::Number(number("30"))));
    let no_amount = condition(amount, FieldComparison::Absent, None);

    assert_eq!(ids(&with_field(expensive)), set(&["invoice"]));
    assert_eq!(ids(&with_field(due_before)), set(&["invoice"]));
    assert_eq!(ids(&with_field(due_soon)), set(&["invoice"]));
    assert_eq!(ids(&with_field(no_amount)), set(&["photo"]));
}

#[test]
fn values_of_different_types_never_match() {
    let wrong_type =
        condition(FieldPresets::amount().key, FieldComparison::Equals, Some(FieldValue::Date(date(2026, 1, 1))));

    assert!(ids(&with_field(wrong_type)).is_empty());
}

#[test]
fn added_since_uses_the_time_a_document_was_added() {
    let query = DocumentQuery { added_since: Some(instant("2024-02-01T00:00:00Z")), ..DocumentQuery::default() };

    assert!(ids(&query).is_empty());
}

#[test]
fn conditions_combine_and_the_content_term_is_left_to_the_content_search() {
    let query = DocumentQuery {
        tags: TagFilter { any: indexset! { tax() }, ..TagFilter::default() },
        category: Some(insurance().id),
        content_text: Some("anything".to_owned()),
        ..DocumentQuery::default()
    };

    assert_eq!(ids(&query), set(&["policy"]));
    assert!(query.needs_content());
}

#[test]
fn a_query_naming_a_category_tag_or_field_the_catalog_lacks_refers_to_something_missing() {
    let catalog = catalog();
    let known = DocumentQuery {
        category: Some(car().id),
        tags: TagFilter { any: indexset! { tax() }, ..TagFilter::default() },
        ..DocumentQuery::default()
    };
    let unknown_field = condition(FieldKey::new("gone").expect("valid"), FieldComparison::Present, None);

    assert!(!known.refers_to_missing(&catalog));
    assert!(
        DocumentQuery { category: Some(category_id("removed")), ..DocumentQuery::default() }
            .refers_to_missing(&catalog)
    );
    assert!(
        with_tags(TagFilter { none: indexset! { tag("gone") }, ..TagFilter::default() }).refers_to_missing(&catalog)
    );
    assert!(with_field(unknown_field).refers_to_missing(&catalog));
}

#[test]
fn a_query_cannot_ask_for_a_category_and_none() {
    let query = DocumentQuery { category: Some(car().id), uncategorized: true, ..DocumentQuery::default() };

    assert!(query.validate().is_err());
}
