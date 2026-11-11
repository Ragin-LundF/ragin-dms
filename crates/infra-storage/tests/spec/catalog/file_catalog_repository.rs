use std::fs;

use dms_application::port::storage::{CatalogLoad, CatalogRepository, StorageFailure};
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::document::{ClassificationRecord, ClassificationSource, DocumentId, DocumentMetadata};
use dms_domain::taxonomy::{Category, CategoryId, TagName};
use indexmap::IndexSet;

use super::{Library, document, instant};
use crate::support::{Applied, failure_of, fixture, folder, write};

fn add_tag(catalog: &Catalog, name: &str) -> CatalogUpdate {
    catalog.set_tags(&DocumentId::new("doc-a").expect("id"), &IndexSet::from([TagName::of(name).expect("tag")]))
}

pub fn sample_catalog() -> Catalog {
    let category = Category::new(CategoryId::new("cat-1").expect("id"), "Invoices", None, None).expect("category");
    let mut later = document("doc-b", "letters/b.pdf", "beta");
    later.added_at = instant("2024-02-01T09:30:00Z");
    later.tags = IndexSet::from([TagName::of("Tax").expect("tag"), TagName::of("2024").expect("tag")]);
    let mut earlier = document("doc-a", "a.pdf", "alpha");
    earlier.metadata = DocumentMetadata::new(
        Some("Sample invoice".to_owned()),
        dms_domain::parse_iso_date("2023-12-24"),
        Some("Example Supplier".to_owned()),
    )
    .expect("metadata");
    earlier.category_id = Some(category.id.clone());
    earlier.classification = Some(ClassificationRecord {
        source: ClassificationSource::Ai,
        model: Some("test-model".to_owned()),
        confirmed_at: instant("2024-01-02T08:00:00Z"),
    });
    Catalog::empty(LibraryId::new("library-1").expect("id"))
        .add_category(category)
        .applied()
        .add_document(later)
        .applied()
        .add_document(earlier)
        .applied()
}

#[test]
fn a_saved_catalog_loads_unchanged() {
    let library = Library::new();
    let catalog = sample_catalog();

    library.save(&catalog);

    assert_eq!(library.repository.load(&library.folder), CatalogLoad::Found { catalog, conflict_copies: Vec::new() });
}

#[test]
fn saving_the_same_catalog_twice_writes_the_same_sorted_bytes() {
    let library = Library::new();
    let catalog = sample_catalog();

    library.save(&catalog);
    let first = library.text();
    library.save(&catalog);

    assert_eq!(library.text(), first);
    assert!(first.find("path: a.pdf") < first.find("path: letters/b.pdf"));
    assert_eq!(first, fixture("catalog/format-1-v1.0.yaml"));
}

#[test]
fn fields_of_a_newer_version_are_kept_when_saving() {
    let library = Library::new();
    let loaded = library.load_text(&fixture("catalog/future-fields.yaml"));
    let metadata = DocumentMetadata::new(Some("Changed".to_owned()), None, None).expect("metadata");

    library.save(&loaded.update_metadata(&DocumentId::new("doc-1").expect("id"), metadata).applied());

    let text = library.text();
    for kept in [
        "futureField: kept at the top",
        "futureNote: kept in the document",
        "futureLayout: kept in the layout",
        "futureInbox: kept in the inbox",
        "futureDefinition: kept in the field definition",
        "futureSearch: kept in the saved search",
        "futureQuery: kept in the query",
        "futureCorrespondent: kept in the correspondent",
        "futureRule: kept in the rule",
        "futureAction: kept in the actions",
        "futureRetention: kept in the retention",
        "futureLink: kept in the link",
    ] {
        assert!(text.contains(kept), "{kept}");
    }
    assert!(text.contains("title: Changed"));
}

#[test]
fn several_fields_of_a_newer_version_keep_their_order() {
    let library = Library::new();
    let text = "formatVersion: 1\nlibraryId: l\nrevision: 1\nfutureB: 2\nfutureA: \"x\"\nfutureC: [1, \"2\"]\n";
    let catalog = library.load_text(text);

    library.save(&catalog);

    assert_eq!(
        library.text(),
        "formatVersion: 1\nlibraryId: l\nrevision: 1\nfutureB: 2\nfutureA: x\nfutureC:\n- 1\n- \"2\"\n"
    );
}

#[test]
fn a_catalog_based_on_an_older_revision_does_not_overwrite_a_newer_file() {
    let library = Library::new();
    let base = sample_catalog();
    library.save(&add_tag(&base, "other device").applied());
    let written = library.text();
    let stale = add_tag(&base, "this device").applied();

    let result = library.repository.save(&library.folder, &stale, base.revision());

    assert_eq!(failure_of(result), Some(StorageFailure::CatalogChanged));
    assert_eq!(library.text(), written);
}

#[test]
fn a_catalog_based_on_the_revision_on_disk_is_written() {
    let library = Library::new();
    let base = sample_catalog();
    library.save(&base);
    let next = add_tag(&base, "this device").applied();

    library.repository.save(&library.folder, &next, base.revision()).expect("saved");

    assert_eq!(library.found(), next);
}

#[test]
fn a_catalog_of_a_newer_format_is_neither_read_nor_overwritten() {
    let library = Library::new();
    let newer = fixture("catalog/future-fields.yaml").replace("formatVersion: 1", "formatVersion: 99");
    fs::write(library.file(), &newer).expect("file");

    let load = library.repository.load(&library.folder);
    let save = library.repository.save(&library.folder, &sample_catalog(), sample_catalog().revision());

    assert!(matches!(load, CatalogLoad::Failed(error) if error.reason == StorageFailure::UnsupportedFormat));
    assert_eq!(failure_of(save), Some(StorageFailure::UnsupportedFormat));
    assert_eq!(library.text(), newer);
}

#[test]
fn a_broken_catalog_file_is_not_overwritten() {
    for broken in [
        "formatVersion: [1\n",
        "formatVersion: 1\nlibraryId: l\n",
        "",
        "formatVersion: 1\nlibraryId: l\nrevision: 1\ndocuments: x\n",
    ] {
        let library = Library::new();
        fs::write(library.file(), broken).expect("file");

        let load = library.repository.load(&library.folder);
        let save = library.repository.save(&library.folder, &sample_catalog(), sample_catalog().revision());

        assert!(
            matches!(load, CatalogLoad::Failed(error) if error.reason == StorageFailure::UnsupportedFormat),
            "{broken}"
        );
        assert!(save.is_err(), "{broken}");
        assert_eq!(library.text(), broken);
    }
}

#[test]
fn a_file_with_an_unreadable_date_fails_instead_of_crashing() {
    let library = Library::new();
    let broken_date = fixture("catalog/format-1-v1.0.yaml").replace("addedAt: 2024-01-01T10:00:00Z", "addedAt: soon");
    fs::write(library.file(), &broken_date).expect("file");

    let load = library.repository.load(&library.folder);

    assert!(matches!(load, CatalogLoad::Failed(error) if error.reason == StorageFailure::UnsupportedFormat));
}

#[test]
fn readable_conflict_copies_are_loaded_and_discarded_unreadable_ones_stay() {
    let library = Library::new();
    let older = sample_catalog();
    library.save(&older);
    fs::copy(library.file(), library.root.join("dms-catalog 2.yaml")).expect("copy");
    let newer = add_tag(&older, "extra").applied();
    library.save(&newer);
    write(&library.root.join("dms-catalog (conflicted copy).yaml"), "revision: [");

    let load = library.repository.load(&library.folder);
    library.repository.discard_conflict_copies(&library.folder).expect("discarded");

    assert_eq!(load, CatalogLoad::Found { catalog: newer, conflict_copies: vec![older] });
    assert_eq!(library.trash.trashed(), ["dms-catalog 2.yaml"]);
    assert!(library.root.join("dms-catalog (conflicted copy).yaml").exists());
}

#[test]
fn a_folder_without_catalog_file_reports_not_found_and_an_offline_folder_fails() {
    let library = Library::new();
    let offline = folder(&library.temp.path().join("offline"));

    assert_eq!(library.repository.load(&library.folder), CatalogLoad::NotFound);
    assert!(matches!(
        library.repository.load(&offline),
        CatalogLoad::Failed(error) if error.reason == StorageFailure::FolderUnreachable
    ));
}
