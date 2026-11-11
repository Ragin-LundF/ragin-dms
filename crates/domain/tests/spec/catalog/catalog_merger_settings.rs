use dms_domain::catalog::{Catalog, CatalogMergeConflict, CatalogMerger};
use dms_domain::document::{Document, DocumentId};
use dms_domain::field::{FieldDefinition, FieldPresets};
use dms_domain::link::DocumentLink;
use dms_domain::query::{DocumentQuery, SavedSearch, SavedSearchId};
use dms_domain::rule::Correspondent;
use dms_domain::settings::LibraryLayout;
use indexmap::{IndexSet, indexset};

use crate::test_data::{self, Applied};

struct Fixture {
    invoice: Document,
    reminder: Document,
    receipt: Document,
    base: Catalog,
}

fn fixture() -> Fixture {
    let invoice = test_data::document("invoice", "invoice.pdf");
    let reminder = test_data::document("reminder", "reminder.pdf");
    let receipt = test_data::document("receipt", "receipt.pdf");
    let base = test_data::catalog_with(&[invoice.clone(), reminder.clone(), receipt.clone()]);
    Fixture { invoice, reminder, receipt, base }
}

fn link(first: &DocumentId, second: &DocumentId, label: Option<&str>) -> DocumentLink {
    DocumentLink::new(first.clone(), second.clone(), label).expect("valid link")
}

#[test]
fn links_of_both_copies_are_united_by_pair() {
    let f = fixture();
    let left = f.base.link(link(&f.invoice.id, &f.reminder.id, None)).applied();
    let right = f.base.link(link(&f.receipt.id, &f.invoice.id, None)).applied();

    let merged = CatalogMerger::merge(&left, &right).expect("same library").catalog;

    assert_eq!(merged.links().len(), 2);
}

#[test]
fn the_newer_copys_label_wins_for_the_same_pair() {
    let f = fixture();
    let older = f.base.link(link(&f.invoice.id, &f.reminder.id, Some("old"))).applied();
    let newer = older.link(link(&f.invoice.id, &f.reminder.id, Some("new"))).applied();

    let merged = CatalogMerger::merge(&newer, &older).expect("same library").catalog;

    assert_eq!(merged.links().len(), 1);
    assert_eq!(merged.links()[0].label(), Some("new"));
}

#[test]
fn field_definitions_are_merged_per_key_and_a_clash_is_reported() {
    let f = fixture();
    let older = f.base.change_settings(f.base.settings().with_field(FieldPresets::amount())).applied();
    let renamed = FieldDefinition { label: "Total".to_owned(), ..FieldPresets::amount() };
    let newer = older
        .change_settings(older.settings().with_field(renamed.clone()).with_field(FieldPresets::due_date()))
        .applied();

    let result = CatalogMerger::merge(&older, &newer).expect("same library");

    assert_eq!(result.catalog.settings().field_definitions, vec![renamed, FieldPresets::due_date()]);
    let [CatalogMergeConflict::FieldDefinitionDiffers { discarded, .. }] = result.conflicts.as_slice() else {
        panic!("expected one field definition conflict, got {:?}", result.conflicts);
    };
    assert_eq!(discarded, &FieldPresets::amount());
}

#[test]
fn saved_searches_only_in_the_older_copy_are_kept() {
    let f = fixture();
    let search = SavedSearch::new(
        SavedSearchId::new("s1").expect("valid"),
        "Untagged",
        DocumentQuery { untagged: true, ..DocumentQuery::default() },
    )
    .expect("valid search");
    let older = f.base.change_settings(f.base.settings().with_search(search.clone())).applied();
    let newer = f.base.set_note(&f.invoice.id, Some("n")).applied().set_note(&f.invoice.id, Some("m")).applied();

    let result = CatalogMerger::merge(&older, &newer).expect("same library");

    assert_eq!(result.catalog.settings().saved_searches, vec![search]);
    assert!(!result.conflicts.iter().any(|it| matches!(it, CatalogMergeConflict::SavedSearchDiffers { .. })));
}

#[test]
fn correspondents_are_united_by_name_with_all_aliases() {
    let f = fixture();
    let left =
        f.base.change_settings(f.base.settings().with_correspondent(correspondent("Acme", "ACME Ltd"))).applied();
    let right =
        f.base.change_settings(f.base.settings().with_correspondent(correspondent("acme", "Acme Inc"))).applied();

    let merged = CatalogMerger::merge(&left, &right).expect("same library").catalog;

    let [only] = merged.settings().correspondents.as_slice() else {
        panic!("expected one correspondent");
    };
    let aliases: IndexSet<&str> = only.aliases.iter().map(String::as_str).collect();
    assert_eq!(aliases, indexset! { "ACME Ltd", "Acme Inc" });
}

#[test]
fn the_newer_layout_wins() {
    let f = fixture();
    let older = f.base.change_settings(f.base.settings().with_layout(layout("{year}"))).applied();
    let newer = older.change_settings(older.settings().with_layout(layout("{category}"))).applied();

    let merged = CatalogMerger::merge(&older, &newer).expect("same library").catalog;

    assert_eq!(merged.settings().layout.path_template.as_deref(), Some("{category}"));
}

fn correspondent(name: &str, alias: &str) -> Correspondent {
    Correspondent::new(name, vec![alias.to_owned()]).expect("valid correspondent")
}

fn layout(template: &str) -> LibraryLayout {
    LibraryLayout { path_template: Some(template.to_owned()), ..LibraryLayout::default() }
}
