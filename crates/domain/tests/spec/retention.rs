use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::retention::{Retention, RetentionDates};
use dms_domain::taxonomy::Category;

use crate::test_data::{self, Applied, category_id, date, instant};

fn tax() -> Category {
    Category::new(category_id("tax"), "Tax", None, None).expect("valid")
}

fn returns() -> Category {
    Category::new(category_id("returns"), "Returns", Some(tax().id), None).expect("valid")
}

fn document() -> Document {
    Document { category_id: Some(returns().id), ..test_data::document("return", "return.pdf") }
}

fn catalog() -> Catalog {
    test_data::catalog_with(&[])
        .add_category(tax())
        .applied()
        .add_category(returns())
        .applied()
        .add_document(document())
        .applied()
        .update_metadata(
            &document().id,
            DocumentMetadata { document_date: Some(date(2020, 4, 15)), ..DocumentMetadata::default() },
        )
        .applied()
}

fn due_date(catalog: &Catalog) -> Option<NaiveDate> {
    RetentionDates::due_date(&catalog.documents()[&document().id], catalog)
}

#[test]
fn nothing_has_a_retention_unless_the_user_sets_one() {
    assert_eq!(due_date(&catalog()), None);
}

#[test]
fn a_category_period_counts_from_the_end_of_the_document_dates_year_inherited_by_subcategories() {
    let with_period = catalog().set_retention_years(&tax().id, Some(10)).applied();

    assert_eq!(due_date(&with_period), Some(date(2030, 12, 31)));
}

#[test]
fn the_documents_own_keep_until_date_wins_over_the_category() {
    let with_both = catalog()
        .set_retention_years(&tax().id, Some(10))
        .applied()
        .set_retention(&document().id, Some(Retention { keep_until: Some(date(2027, 1, 31)), settled_at: None }))
        .applied();

    assert_eq!(due_date(&with_both), Some(date(2027, 1, 31)));
}

#[test]
fn a_settled_retention_is_never_flagged_again_also_not_through_the_category() {
    let settled = catalog()
        .set_retention_years(&tax().id, Some(1))
        .applied()
        .set_retention(
            &document().id,
            Some(Retention { keep_until: None, settled_at: Some(instant("2026-01-01T00:00:00Z")) }),
        )
        .applied();

    assert_eq!(due_date(&settled), None);
}
