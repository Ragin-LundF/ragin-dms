use dms_application::common::OperationOutcome;
use dms_application::port::state::SettingsRepository;
use dms_application::port::storage::StorageFailure;
use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::{Applied, with_revision};

fn mine() -> Document {
    TestLibrary::document("mine", "letters/mine.pdf", "mine")
}

fn theirs() -> Document {
    TestLibrary::document("theirs", "letters/theirs.pdf", "theirs")
}

fn theirs_too() -> Document {
    TestLibrary::document("theirs-2", "letters/theirs-2.pdf", "theirs 2")
}

fn added(catalog: &Catalog, document: Document) -> Catalog {
    catalog.add_document(document).applied()
}

#[test]
fn a_newer_catalog_another_device_saved_is_merged_and_the_edit_is_applied_on_top() {
    let library = TestLibrary::new();
    let opened = library.open();
    library.folders.set_catalog(&library.primary, added(&opened, theirs()));

    let outcome = library.store.commit(|catalog| catalog.add_document(mine()));

    let OperationOutcome::Done { catalog, .. } = outcome else { panic!("expected Done, got {outcome:?}") };
    let ids: IndexSet<_> = catalog.documents().keys().cloned().collect();
    assert_eq!(ids, IndexSet::from([mine().id, theirs().id]));
    for folder in library.all_folders() {
        assert_eq!(library.folders.catalog_in(&folder).as_ref(), Some(&catalog));
    }
    assert_eq!(library.catalog(), catalog);
}

#[test]
fn a_second_refused_save_is_reported_and_the_newer_catalog_stays() {
    let library = TestLibrary::new();
    let opened = library.open();
    let folders = library.folders.clone();
    let primary = library.primary.clone();
    library.folders.set_before_save(move |folder| {
        if *folder == primary {
            let on_disk = folders.catalog_in(folder).expect("a catalog on disk");
            folders.set_catalog(folder, with_revision(&on_disk, on_disk.revision().next().next()));
        }
    });

    let outcome = library.store.commit(|catalog| catalog.add_document(mine()));

    let OperationOutcome::StorageFailed(failure) = outcome else { panic!("expected StorageFailed, got {outcome:?}") };
    assert_eq!(failure.reason, StorageFailure::CatalogChanged);
    let primary = library.folders.catalog_in(&library.primary).expect("a catalog on disk");
    assert!(!primary.documents().contains_key(&mine().id));
    assert!(primary.revision() > opened.revision());
}

#[test]
fn a_sync_folder_holding_a_newer_catalog_is_not_overwritten() {
    let library = TestLibrary::new();
    let opened = library.open();
    let newer = added(&added(&opened, theirs()), theirs_too());
    library.folders.set_catalog(&library.sync_a, newer.clone());

    let outcome = library.store.commit(|catalog| catalog.add_document(mine()));

    let OperationOutcome::Done { catalog, pending_folders } = outcome else { panic!("expected Done, got {outcome:?}") };
    assert_eq!(pending_folders, IndexSet::from([library.sync_a.id.clone()]));
    assert_eq!(library.folders.catalog_in(&library.sync_a), Some(newer));
    assert_eq!(library.folders.catalog_in(&library.sync_b), Some(catalog));
}

#[test]
fn a_foreign_folder_made_primary_before_the_library_is_opened_again_reads_as_not_open() {
    let library = TestLibrary::new();
    let opened = library.open();
    library.store.publish(opened, IndexSet::from([library.sync_a.id.clone()]));
    let folders = library.settings.load_library_folders().expect("settings").expect("folders");
    library
        .settings
        .save_library_folders(&folders.with_primary(&library.sync_a.id).expect("valid primary"))
        .expect("saved");

    assert!(library.store.current().is_none());
    let outcome = library.store.commit(|catalog| catalog.add_document(mine()));
    assert_eq!(outcome, OperationOutcome::LibraryNotOpen);
}
