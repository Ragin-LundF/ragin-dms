use dms_application::common::OperationOutcome;
use dms_application::port::storage::DocumentStorage;

use crate::support::TestLibrary;

#[test]
fn nothing_is_trashed_without_confirmation() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");

    let outcome = library.trash_document().run(&document.id, false);

    assert_eq!(outcome, OperationOutcome::NotConfirmed);
    assert_eq!(library.folders.content(&library.primary, "letters/a.pdf").as_deref(), Some("alpha"));
}

#[test]
fn a_confirmed_document_goes_to_the_trash_in_every_folder_and_leaves_the_catalog() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");
    // Already gone from one sync folder: that folder has nothing left to do.
    library.folders.move_to_trash(&library.sync_a, &document.path).expect("trashed");

    let outcome = library.trash_document().run(&document.id, true);

    let OperationOutcome::Done { pending_folders, .. } = outcome else { panic!("expected Done, got {outcome:?}") };
    assert!(pending_folders.is_empty());
    for folder in library.all_folders() {
        assert!(library.folders.paths(&folder).is_empty());
    }
    assert!(library.catalog().documents().is_empty());
}
