use chrono_tz::Tz;
use dms_application::export::{ExportOptions, ExportPlanner};
use dms_application::port::export::{ExportBundle, IndexRow};
use dms_domain::catalog::{Catalog, CatalogRevision};
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::field::{Decimal, FieldDefinition, FieldKey, FieldType, FieldValue};
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::Category;
use indexmap::{IndexMap, IndexSet};

use crate::support::TestLibrary;
use crate::support::test_data::{category_id, date, instant, library_id, tag};

fn finance() -> Category {
    Category::new(category_id("finance"), "Finance", None, None).expect("valid category")
}

fn bills() -> Category {
    Category::new(category_id("bills"), "Bills", Some(category_id("finance")), None).expect("valid category")
}

fn amount() -> FieldDefinition {
    FieldDefinition::new(FieldKey::new("amount").expect("valid key"), "Amount", FieldType::Number, None)
        .expect("valid definition")
}

fn due() -> FieldDefinition {
    FieldDefinition::new(FieldKey::new("due").expect("valid key"), "Due", FieldType::Date, None).expect("valid")
}

fn catalog() -> Catalog {
    Catalog::new(
        library_id("library"),
        CatalogRevision::INITIAL,
        IndexMap::new(),
        IndexMap::from([(finance().id, finance()), (bills().id, bills())]),
        IndexSet::new(),
        IndexSet::new(),
        CatalogSettings { field_definitions: vec![amount(), due()], ..CatalogSettings::default() },
    )
    .expect("valid catalog")
}

fn plan(documents: &[Document], options: ExportOptions) -> ExportBundle {
    ExportPlanner::plan(&catalog(), documents, options, Tz::UTC)
}

fn document(id: &str, path: &str, title: Option<&str>) -> Document {
    let document = TestLibrary::document(id, path, id);
    Document {
        metadata: DocumentMetadata { title: title.map(str::to_owned), ..DocumentMetadata::default() },
        ..document
    }
}

fn targets(bundle: &ExportBundle) -> Vec<&str> {
    bundle.entries.iter().map(|entry| entry.target.as_str()).collect()
}

#[test]
fn keeps_the_folders_and_sorts_the_files_by_their_path() {
    let documents =
        [document("1", "2025/scan.pdf", None), document("2", "2024/scan.pdf", None), document("3", "top.pdf", None)];

    let bundle = plan(&documents, ExportOptions::default());

    let pairs: Vec<_> = bundle.entries.iter().map(|entry| (entry.source.value(), entry.target.as_str())).collect();
    assert_eq!(
        pairs,
        vec![("2024/scan.pdf", "2024/scan.pdf"), ("2025/scan.pdf", "2025/scan.pdf"), ("top.pdf", "top.pdf")]
    );
}

#[test]
fn flattening_puts_every_file_at_the_top_and_suffixes_clashes_in_path_order_ignoring_case() {
    let documents =
        [document("1", "b/Scan.pdf", None), document("2", "a/scan.pdf", None), document("3", "c/SCAN.pdf", None)];

    let bundle = plan(&documents, ExportOptions { flatten: true, ..ExportOptions::default() });

    assert_eq!(targets(&bundle), vec!["scan.pdf", "Scan (2).pdf", "SCAN (3).pdf"]);
}

#[test]
fn folders_that_differ_only_by_case_share_their_names() {
    let documents = [document("1", "Letters/scan.pdf", None), document("2", "letters/Scan.pdf", None)];

    let bundle = plan(&documents, ExportOptions::default());

    assert_eq!(targets(&bundle), vec!["Letters/scan.pdf", "letters/Scan (2).pdf"]);
}

#[test]
fn renaming_to_the_title_makes_it_a_valid_name_and_keeps_the_extension() {
    let documents = [
        document("1", "letters/scan.pdf", Some("Power: bill?")),
        document("2", "letters/untitled.pdf", None),
        document("3", "letters/notes", Some("Plain notes")),
    ];

    let bundle = plan(&documents, ExportOptions { rename_to_title: true, ..ExportOptions::default() });

    assert_eq!(targets(&bundle), vec!["letters/Plain notes", "letters/Power- bill-.pdf", "letters/untitled.pdf"]);
}

#[test]
fn a_long_title_is_cut_so_that_the_name_with_its_extension_stays_within_the_limit() {
    let title = "x".repeat(300);
    let documents = [document("1", "scan.pdf", Some(&title))];

    let bundle = plan(&documents, ExportOptions { rename_to_title: true, ..ExportOptions::default() });

    assert_eq!(targets(&bundle), vec![format!("{}.pdf", "x".repeat(196))]);
}

#[test]
fn documents_with_the_same_title_get_a_suffix() {
    let documents = [document("1", "letters/a.pdf", Some("Bill")), document("2", "letters/b.pdf", Some("bill"))];

    let bundle = plan(&documents, ExportOptions { rename_to_title: true, ..ExportOptions::default() });

    assert_eq!(targets(&bundle), vec!["letters/Bill.pdf", "letters/bill (2).pdf"]);
}

#[test]
fn the_index_describes_every_file_under_its_exported_path() {
    let documents = [Document {
        metadata: DocumentMetadata {
            title: Some("Power bill".to_owned()),
            document_date: Some(date(2024, 2, 1)),
            correspondent: Some("Sample Energy".to_owned()),
        },
        category_id: Some(bills().id),
        tags: IndexSet::from([tag("tax"), tag("Energy")]),
        fields: IndexMap::from([(due().key, FieldValue::Date(date(2024, 3, 1)))]),
        note: Some("paid".to_owned()),
        ..document("1", "letters/scan.pdf", Some("Power bill"))
    }];

    let bundle = plan(&documents, ExportOptions { flatten: true, ..ExportOptions::default() });

    assert_eq!(
        bundle.index,
        Some(vec![IndexRow {
            path: "scan.pdf".to_owned(),
            title: "Power bill".to_owned(),
            date: "2024-02-01".to_owned(),
            correspondent: "Sample Energy".to_owned(),
            category: "Finance / Bills".to_owned(),
            tags: "Energy, tax".to_owned(),
            fields: vec![String::new(), "2024-03-01".to_owned()],
            note: "paid".to_owned(),
        }])
    );
    assert_eq!(bundle.field_labels, vec!["Amount", "Due"]);
}

#[test]
fn a_document_without_details_has_empty_columns_and_the_day_it_was_added_in_the_given_zone() {
    let documents = [Document {
        added_at: instant("2024-01-01T23:30:00Z"),
        fields: IndexMap::from([(amount().key, FieldValue::Number(Decimal::parse("12.50").expect("a number")))]),
        ..document("1", "scan.pdf", None)
    }];

    let bundle = ExportPlanner::plan(&catalog(), &documents, ExportOptions::default(), Tz::Europe__Berlin);

    assert_eq!(
        bundle.index,
        Some(vec![IndexRow {
            path: "scan.pdf".to_owned(),
            title: String::new(),
            date: "2024-01-02".to_owned(),
            correspondent: String::new(),
            category: String::new(),
            tags: String::new(),
            fields: vec!["12.50".to_owned(), String::new()],
            note: String::new(),
        }])
    );
}

#[test]
fn a_category_the_catalog_does_not_know_leaves_the_column_empty() {
    let documents = [Document { category_id: Some(category_id("gone")), ..document("1", "scan.pdf", None) }];

    let bundle = plan(&documents, ExportOptions::default());

    let categories: Option<Vec<String>> = bundle.index.map(|rows| rows.into_iter().map(|row| row.category).collect());
    assert_eq!(categories, Some(vec![String::new()]));
}

#[test]
fn without_an_index_the_bundle_holds_only_the_files() {
    let bundle = plan(&[document("1", "scan.pdf", None)], ExportOptions { index: false, ..ExportOptions::default() });

    assert_eq!(bundle.index, None);
    assert_eq!(targets(&bundle), vec!["scan.pdf"]);
}

#[test]
fn no_documents_make_an_empty_bundle() {
    let bundle = plan(&[], ExportOptions::default());

    assert_eq!(bundle.entries, Vec::new());
    assert_eq!(bundle.index, Some(Vec::new()));
}
