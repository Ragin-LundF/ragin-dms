use dms_application::common::OperationOutcome;
use dms_application::document::AddDocumentsOutcome;
use dms_application::port::storage::{OperationJournal, StorageFailure};
use dms_domain::library::{FileNameViolation, RelativePath};
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::fake::storage::FakeFolders;
use crate::support::test_data::path;

fn letters() -> RelativePath {
    path("letters")
}

#[test]
fn a_file_is_copied_into_every_folder_and_catalogued() {
    let library = TestLibrary::new();
    library.open();

    let document = library.add("letters/a.pdf", "alpha");

    assert_eq!(document.content_hash, FakeFolders::hash_of("alpha"));
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "letters/a.pdf").as_deref(), Some("alpha"));
        assert_eq!(library.folders.catalog_in(&folder), Some(library.catalog()));
    }
}

#[test]
fn a_name_that_is_taken_ignoring_case_gets_a_suffix() {
    let library = TestLibrary::new();
    library.open();
    library.add("letters/a.pdf", "alpha");
    let source = library.folders.external("/outside/A.pdf", "beta");

    let added = library.added(&[source], &letters()).pop().expect("one file");

    assert_eq!(added.path, Some(path("letters/A (2).pdf")));
    assert_eq!(library.folders.content(&library.sync_a, "letters/a.pdf").as_deref(), Some("alpha"));
}

#[test]
fn a_name_the_file_name_policy_refuses_is_not_added() {
    let library = TestLibrary::new();
    library.open();
    let source = library.folders.external("/outside/con.pdf", "x");

    let added = library.added(&[source], &letters()).pop().expect("one file");

    assert_eq!(added.outcome, OperationOutcome::InvalidName(FileNameViolation::ReservedName));
    assert!(library.folders.paths(&library.primary).is_empty());
}

#[test]
fn an_unreachable_sync_folder_is_journaled_and_reported_as_pending() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.sync_b);
    let source = library.folders.external("/outside/a.pdf", "alpha");

    let added = library.added(&[source], &letters()).pop().expect("one file");

    let OperationOutcome::Done { pending_folders, .. } = added.outcome else { panic!("expected Done") };
    assert_eq!(pending_folders, IndexSet::from([library.sync_b.id.clone()]));
    assert_eq!(library.journal.pending().expect("journal").len(), 2);
    assert_eq!(library.folders.content(&library.sync_a, "letters/a.pdf").as_deref(), Some("alpha"));
}

#[test]
fn an_unreachable_primary_folder_changes_nothing() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.primary);
    let source = library.folders.external("/outside/a.pdf", "alpha");

    let added = library.added(&[source], &letters()).pop().expect("one file");

    let OperationOutcome::StorageFailed(failure) = added.outcome else { panic!("expected StorageFailed") };
    assert_eq!(failure.reason, StorageFailure::FolderUnreachable);
    assert!(library.folders.paths(&library.sync_a).is_empty());
    assert!(library.catalog().documents().is_empty());
}

#[test]
fn content_the_library_already_has_is_added_and_reported_as_a_duplicate() {
    let library = TestLibrary::new();
    library.open();
    let existing = library.add("letters/a.pdf", "alpha");
    let source = library.folders.external("/outside/b.pdf", "alpha");

    let added = library.added(&[source], &letters()).pop().expect("one file");

    assert!(matches!(added.outcome, OperationOutcome::Done { .. }));
    assert_eq!(added.duplicates, vec![existing]);
}

#[test]
fn nothing_is_added_while_no_library_is_open() {
    let library = TestLibrary::new();
    let source = library.folders.external("/outside/a.pdf", "a");

    let outcome = library.add_documents().run(&[source], &RelativePath::root());

    assert_eq!(outcome, AddDocumentsOutcome::LibraryNotOpen);
}
