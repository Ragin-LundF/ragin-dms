use dms_domain::catalog::{Catalog, CatalogMergeConflict, CatalogMerger, LibraryId};
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::taxonomy::Category;
use indexmap::{IndexSet, indexset};

use crate::test_data::{self, Applied, category_id};

fn shared() -> Document {
    test_data::document("shared", "shared.pdf")
}

fn base() -> Catalog {
    test_data::catalog_with(&[shared()])
}

fn titled(title: &str) -> DocumentMetadata {
    DocumentMetadata { title: Some(title.to_owned()), ..DocumentMetadata::default() }
}

#[test]
fn identical_copies_merge_without_a_new_revision() {
    let base = base();

    let result = CatalogMerger::merge(&base, &base.clone()).expect("same library");

    assert_eq!(result.catalog, base);
    assert_eq!(result.catalog.revision(), base.revision());
    assert_eq!(result.conflicts, [] as [dms_domain::catalog::CatalogMergeConflict; 0]);
}

#[test]
fn entries_added_on_either_side_are_kept_and_nothing_is_deleted() {
    let left = base().add_document(test_data::document("left", "left.pdf")).applied();
    let right = base().add_document(test_data::document("right", "right.pdf")).applied();

    let result = CatalogMerger::merge(&left, &right).expect("same library");

    let ids: IndexSet<&str> = result.catalog.documents().keys().map(dms_domain::document::DocumentId::value).collect();
    assert_eq!(ids, indexset! { "shared", "left", "right" });
    assert_eq!(result.catalog.revision(), left.revision().next());
    assert_eq!(result.conflicts, [] as [dms_domain::catalog::CatalogMergeConflict; 0]);
}

#[test]
fn the_copy_with_the_higher_revision_wins_a_differing_document() {
    let older = base().update_metadata(&shared().id, titled("Old")).applied();
    let newer = older.update_metadata(&shared().id, titled("New")).applied();

    let result = CatalogMerger::merge(&older, &newer).expect("same library");

    assert_eq!(result.catalog.documents()[&shared().id].metadata.title.as_deref(), Some("New"));
    let [CatalogMergeConflict::DocumentDiffers { discarded, .. }] = result.conflicts.as_slice() else {
        panic!("expected one differing document, got {:?}", result.conflicts);
    };
    assert_eq!(discarded.metadata.title.as_deref(), Some("Old"));
}

#[test]
fn two_documents_on_the_same_path_are_reported_as_a_path_clash() {
    let left = base().add_document(test_data::document("left", "same.pdf")).applied();
    let right = left
        .remove_document(&test_data::document("left", "same.pdf").id)
        .applied()
        .add_document(test_data::document("right", "same.pdf"))
        .applied();

    let result = CatalogMerger::merge(&left, &right).expect("same library");

    let [CatalogMergeConflict::PathClash { kept, discarded }] = result.conflicts.as_slice() else {
        panic!("expected one path clash, got {:?}", result.conflicts);
    };
    assert_eq!(kept.id.value(), "right");
    assert_eq!(discarded.id.value(), "left");
}

#[test]
fn categories_are_merged_with_the_newer_copy_winning() {
    let category = Category::new(category_id("c"), "Letters", None, None).expect("valid");
    let older = base().add_category(category.clone()).applied();
    let newer = older
        .rename_category(&category.id, "Mail")
        .applied()
        .add_category(Category::new(category_id("n"), "New", None, None).expect("valid"))
        .applied();

    let result = CatalogMerger::merge(&newer, &older).expect("same library");

    assert_eq!(result.catalog.categories()[&category.id].name, "Mail");
    assert_eq!(result.catalog.categories().len(), 2);
    assert!(matches!(result.conflicts.as_slice(), [CatalogMergeConflict::CategoryDiffers { .. }]));
}

#[test]
fn copies_of_different_libraries_cannot_be_merged() {
    let other = Catalog::empty(LibraryId::new("other").expect("valid"));

    assert!(CatalogMerger::merge(&base(), &other).is_err());
}
