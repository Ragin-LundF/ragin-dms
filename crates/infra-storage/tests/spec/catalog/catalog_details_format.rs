//! The fields added in 1.1: custom fields, notes, retention, links and the library-wide settings.

use dms_domain::catalog::{Catalog, LibraryId};
use dms_domain::field::{CurrencyCode, Decimal, FieldDefinition, FieldKey, FieldPresets, FieldType, FieldValue};
use dms_domain::link::DocumentLink;
use dms_domain::query::{
    DateRange, DocumentQuery, FieldComparison, FieldCondition, SavedSearch, SavedSearchId, TagFilter,
};
use dms_domain::retention::Retention;
use dms_domain::rule::{Correspondent, MatchingRule, MatchingRuleId, RuleActions, RuleCondition, RuleOrigin};
use dms_domain::settings::{CatalogSettings, InboxSettings, LibraryLayout};
use dms_domain::taxonomy::{Category, CategoryId, TagName};
use dms_domain::{parse_iso_date, settings};
use indexmap::{IndexMap, IndexSet};

use super::{Library, document, instant};
use crate::support::{Applied, fixture, hash_of, path};

fn euro() -> CurrencyCode {
    CurrencyCode::new("EUR").expect("currency")
}

fn category() -> Category {
    Category::new(CategoryId::new("cat-1").expect("id"), "Contracts", None, Some(10)).expect("category")
}

fn project() -> FieldDefinition {
    FieldDefinition::new(FieldKey::new("project").expect("key"), "Project", FieldType::Text, None).expect("definition")
}

fn tax() -> TagName {
    TagName::of("Tax").expect("tag")
}

pub fn detailed_catalog() -> Catalog {
    let first = document("doc-a", "a.pdf", "alpha");
    let second = document("doc-b", "letters/b.pdf", "beta");
    let fields = IndexMap::from([
        (
            FieldPresets::amount().key,
            FieldValue::Money { amount: Decimal::parse("1234.50").expect("amount"), currency: euro() },
        ),
        (FieldPresets::cancel_by().key, FieldValue::Date(parse_iso_date("2026-09-30").expect("date"))),
        (project().key, FieldValue::text("Home").expect("text")),
    ]);
    let retention =
        Retention { keep_until: parse_iso_date("2030-12-31"), settled_at: Some(instant("2026-02-01T08:00:00Z")) };
    let link = DocumentLink::new(second.id.clone(), first.id.clone(), Some("reminder")).expect("link");
    Catalog::empty(LibraryId::new("library-1").expect("id"))
        .add_category(category())
        .applied()
        .add_document(first.clone())
        .applied()
        .add_document(second.clone())
        .applied()
        .change_settings(detailed_settings())
        .applied()
        .set_fields(&first.id, fields)
        .applied()
        .set_note(&first.id, Some("Paid on time.\nSecond line"))
        .applied()
        .set_retention(&second.id, Some(retention))
        .applied()
        .link(link)
        .applied()
}

fn detailed_settings() -> CatalogSettings {
    let mut field_definitions = FieldPresets::all();
    field_definitions.push(project());
    CatalogSettings {
        field_definitions,
        saved_searches: vec![saved_search()],
        rules: vec![rule()],
        correspondents: vec![Correspondent::new("Example Energy", vec!["EE Ltd".to_owned()]).expect("correspondent")],
        layout: LibraryLayout {
            inbox: InboxSettings {
                enabled: true,
                path: path("Inbox/New"),
                auto_analyze: true,
                ..InboxSettings::default()
            },
            path_template: Some("{category}/{year}".to_owned()),
            default_currency: Some(euro()),
            embed_metadata: false,
        },
        ..CatalogSettings::default()
    }
}

fn saved_search() -> SavedSearch {
    let query = DocumentQuery {
        text: Some("invoice".to_owned()),
        category: Some(category().id),
        tags: TagFilter {
            all: IndexSet::from([tax()]),
            none: IndexSet::from([TagName::of("Draft").expect("tag")]),
            ..TagFilter::default()
        },
        dates: Some(DateRange::year(2026).expect("year")),
        folder: Some(path("letters")),
        fields: vec![
            FieldCondition::new(
                FieldPresets::amount().key,
                FieldComparison::GreaterThan,
                Some(FieldValue::Number(Decimal::parse("100").expect("number"))),
            )
            .expect("condition"),
            FieldCondition::new(FieldPresets::due_date().key, FieldComparison::Present, None).expect("condition"),
        ],
        added_since: Some(instant("2026-01-01T00:00:00Z")),
        content_text: Some("total".to_owned()),
        ..DocumentQuery::default()
    };
    SavedSearch::new(SavedSearchId::new("search-1").expect("id"), "Tax 2026", query).expect("search")
}

fn rule() -> MatchingRule {
    let actions = RuleActions {
        category_id: Some(category().id),
        tags: IndexSet::from([tax()]),
        fields: IndexMap::from([(project().key, FieldValue::text("Home").expect("text"))]),
        target_folder: Some(path("letters")),
    };
    let conditions = vec![
        RuleCondition::CorrespondentIs { name: "Example Energy".to_owned() },
        RuleCondition::TextContains { words: vec!["meter".to_owned(), "kWh".to_owned()], require_all: true },
        RuleCondition::FileNameMatches { glob: "bill-*.pdf".to_owned() },
    ];
    let mut rule = MatchingRule::new(
        MatchingRuleId::new("rule-1").expect("id"),
        "Example Energy",
        conditions,
        actions,
        RuleOrigin::Learned,
    )
    .expect("rule");
    rule.enabled = false;
    rule
}

#[test]
fn every_new_field_survives_a_round_trip() {
    let library = Library::new();
    let catalog = detailed_catalog();

    library.save(&catalog);

    assert_eq!(library.found(), catalog);
}

#[test]
fn the_detailed_catalog_is_written_as_the_1_1_fixture() {
    let library = Library::new();

    library.save(&detailed_catalog());

    assert_eq!(library.text(), fixture("catalog/format-1-v1.1.yaml"));
}

#[test]
fn a_note_is_written_as_a_block_and_values_as_plain_text() {
    let library = Library::new();

    library.save(&detailed_catalog());

    let text = library.text();
    assert!(text.contains("note: |-\n    Paid on time.\n    Second line"));
    assert!(text.contains("amount: \"1234.50\""));
    assert!(text.contains("currency: EUR"));
}

#[test]
fn a_library_without_the_new_fields_writes_none_of_them() {
    let library = Library::new();

    library.save(&Catalog::empty(LibraryId::new("library-1").expect("id")));

    let text = library.text();
    for absent in ["layout", "fieldDefinitions", "savedSearches", "rules", "correspondents", "links"] {
        assert!(!text.contains(absent), "{absent}");
    }
}

#[test]
fn a_file_of_version_1_0_without_the_new_fields_still_loads() {
    let hash = hash_of("alpha");
    let text = format!(
        "formatVersion: 1\nlibraryId: library-1\nrevision: 2\ncategories:\n- id: cat-1\n  name: Invoices\ndocuments:\n- id: doc-1\n  path: a.pdf\n  contentHash: {}\n  size: 5\n  addedAt: 2024-01-01T10:00:00Z\n  categoryId: cat-1\n",
        hash.value()
    );

    let catalog = Library::new().load_text(&text);

    assert_eq!(catalog.settings(), &settings::CatalogSettings::default());
    assert!(catalog.links().is_empty());
    assert!(catalog.documents().values().next().expect("document").fields.is_empty());
}

#[test]
fn links_to_documents_an_older_version_removed_are_dropped_instead_of_breaking_the_file() {
    let text = format!(
        "formatVersion: 1\nlibraryId: library-1\nrevision: 2\ndocuments:\n- id: doc-1\n  path: a.pdf\n  contentHash: {}\n  size: 5\n  addedAt: 2024-01-01T10:00:00Z\nlinks:\n- first: doc-0\n  second: doc-1\n",
        hash_of("alpha").value()
    );

    assert!(Library::new().load_text(&text).links().is_empty());
}

#[test]
fn unknown_fields_next_to_the_new_ones_are_still_kept() {
    let library = Library::new();
    let catalog = detailed_catalog();
    library.save(&catalog);
    let with_future = library.text().replace("revision: ", "futureSetting: kept\nrevision: ");
    std::fs::write(library.file(), with_future).expect("file");

    library.save(
        &catalog.set_note(&dms_domain::document::DocumentId::new("doc-b").expect("id"), Some("Changed")).applied(),
    );

    let text = library.text();
    assert!(text.contains("futureSetting: kept"));
    assert!(text.contains("note: Changed"));
}
