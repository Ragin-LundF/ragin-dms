use dms_domain::catalog::{Catalog, CatalogRejection, CatalogRevision, CatalogUpdate};
use dms_domain::document::{Document, DocumentMetadata};
use indexmap::{IndexSet, indexset};

use crate::test_data::{self, Applied, category_id, date, document_id, hash, path, tag};

fn invoice() -> Document {
    test_data::document("d1", "projects/alpha/invoice.pdf")
}

fn contract() -> Document {
    test_data::document("d2", "projects/alpha/contract.pdf")
}

fn notes() -> Document {
    test_data::document("d3", "projects/beta/notes.docx")
}

#[test]
fn every_applied_change_increments_the_revision() {
    let catalog = test_data::catalog_with(&[invoice(), contract()]);

    assert_eq!(catalog.revision(), CatalogRevision::new(2));
}

#[test]
fn a_document_id_or_path_can_only_be_used_once() {
    let catalog = test_data::catalog_with(&[invoice()]);

    assert_eq!(
        catalog.add_document(Document { path: path("other.pdf"), ..invoice() }),
        Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentAlreadyExists))
    );
    assert_eq!(
        catalog.add_document(Document { path: invoice().path, ..contract() }),
        Ok(CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed))
    );
}

#[test]
fn a_document_with_an_unknown_category_is_rejected() {
    let document = Document { category_id: Some(category_id("missing")), ..invoice() };

    assert_eq!(
        Catalog::empty(test_data::library_id()).add_document(document),
        Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound))
    );
}

#[test]
fn moving_a_document_onto_another_documents_path_is_rejected() {
    let catalog = test_data::catalog_with(&[invoice(), contract()]);

    assert_eq!(
        catalog.move_document(&invoice().id, contract().path),
        Ok(CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed))
    );
}

#[test]
fn moving_a_document_changes_its_path_and_keeps_its_id() {
    let new_path = path("archive/invoice.pdf");

    let moved = test_data::catalog_with(&[invoice()]).move_document(&invoice().id, new_path.clone()).applied();

    assert_eq!(moved.find_by_path(&new_path).map(|document| &document.id), Some(&invoice().id));
    assert!(moved.find_by_path(&invoice().path).is_none());
}

#[test]
fn moving_a_document_to_the_root_fails_like_the_kotlin_require() {
    let catalog = test_data::catalog_with(&[invoice()]);

    assert!(catalog.move_document(&invoice().id, path("")).is_err());
}

#[test]
fn moving_a_folder_relocates_every_document_below_it() {
    let catalog = test_data::catalog_with(&[invoice(), contract(), notes()]);

    let moved = catalog.move_folder(&path("projects/alpha"), &path("archive/alpha")).applied();

    let paths: IndexSet<&str> = moved.documents().values().map(|document| document.path.value()).collect();
    assert_eq!(
        paths,
        indexset! { "archive/alpha/invoice.pdf", "archive/alpha/contract.pdf", "projects/beta/notes.docx" }
    );
}

#[test]
fn moving_a_folder_into_itself_or_onto_occupied_paths_is_rejected() {
    let catalog = test_data::catalog_with(&[invoice(), Document { path: path("archive/invoice.pdf"), ..notes() }]);
    let alpha = path("projects/alpha");

    assert_eq!(
        catalog.move_folder(&alpha, &path("projects/alpha/sub")),
        CatalogUpdate::Rejected(CatalogRejection::InvalidFolderMove)
    );
    assert_eq!(catalog.move_folder(&path(""), &alpha), CatalogUpdate::Rejected(CatalogRejection::InvalidFolderMove));
    assert_eq!(
        catalog.move_folder(&alpha, &path("archive")),
        CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed)
    );
}

#[test]
fn changes_to_an_unknown_document_are_rejected() {
    let catalog = test_data::catalog_with(&[invoice()]);
    let unknown = document_id("unknown");

    assert_eq!(
        catalog.update_metadata(&unknown, DocumentMetadata::default()),
        Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound))
    );
    assert_eq!(catalog.remove_document(&unknown), CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
}

#[test]
fn metadata_and_content_updates_replace_the_stored_values() {
    let metadata =
        DocumentMetadata::new(Some("Example invoice".to_owned()), Some(date(2024, 1, 15)), None).expect("valid");

    let catalog = test_data::catalog_with(&[invoice()])
        .update_metadata(&invoice().id, metadata.clone())
        .applied()
        .update_content(&invoice().id, hash(42), 7)
        .applied();

    let updated = &catalog.documents()[&invoice().id];
    assert_eq!(updated.metadata, metadata);
    assert_eq!(updated.content_hash, hash(42));
    assert_eq!(updated.size, 7);
}

#[test]
fn setting_tags_registers_new_tags_with_the_catalogs_existing_spelling() {
    let catalog = test_data::catalog_with(&[invoice(), contract()])
        .set_tags(&invoice().id, &indexset! { tag("Tax") })
        .applied()
        .set_tags(&contract().id, &indexset! { tag("tax"), tag("Car") })
        .applied();

    let mut names: Vec<&str> = catalog.tags().iter().map(dms_domain::taxonomy::TagName::value).collect();
    names.sort_unstable();
    assert_eq!(names, vec!["Car", "Tax"]);
    let contract_tags: IndexSet<&str> =
        catalog.documents()[&contract().id].tags.iter().map(dms_domain::taxonomy::TagName::value).collect();
    assert_eq!(contract_tags, indexset! { "Tax", "Car" });
}

#[test]
fn chained_updates_stop_at_the_first_rejection() {
    let catalog = test_data::catalog_with(&[invoice()]);

    let chained =
        catalog.remove_document(&document_id("unknown")).and_then(|catalog| catalog.remove_document(&invoice().id));

    assert_eq!(chained, CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
}

#[test]
fn documents_inside_a_folder_include_nested_folders() {
    let catalog = test_data::catalog_with(&[invoice(), contract(), notes()]);

    let inside: IndexSet<_> =
        catalog.documents_inside(&path("projects/alpha")).iter().map(|document| document.id.clone()).collect();

    assert_eq!(inside, indexset! { invoice().id, contract().id });
}

#[test]
fn documents_can_be_found_by_content_hash() {
    let copy = Document { id: document_id("copy"), path: path("copy.pdf"), ..contract() };
    let catalog = test_data::catalog_with(&[invoice(), contract(), copy.clone()]);

    let found: IndexSet<_> =
        catalog.find_by_content_hash(&contract().content_hash).iter().map(|document| document.id.clone()).collect();

    assert_eq!(found, indexset! { contract().id, copy.id });
}
