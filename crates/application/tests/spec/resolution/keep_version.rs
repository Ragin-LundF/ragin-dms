use dms_application::common::OperationOutcome;
use dms_domain::library::RelativePath;

use crate::support::TestLibrary;
use crate::support::fake::storage::FakeFolders;
use crate::support::test_data::{done, path};

fn conflicted() -> RelativePath {
    path("notes/n.txt")
}

#[test]
fn nothing_is_replaced_without_confirmation() {
    let library = TestLibrary::new();
    library.open();

    let outcome = library.keep_version().run(&conflicted(), &library.sync_a.id, false);

    assert_eq!(outcome, OperationOutcome::NotConfirmed);
}

#[test]
fn the_kept_version_replaces_the_others_which_go_to_the_trash() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.primary, conflicted().value(), "one");
    library.folders.put(&library.sync_a, conflicted().value(), "two");
    library.folders.put(&library.sync_b, conflicted().value(), "three");

    let outcome = library.keep_version().run(&conflicted(), &library.sync_a.id, true);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, conflicted().value()).as_deref(), Some("two"));
    }
    let trashed = library.folders.trashed();
    assert!(trashed.contains(&(library.primary.id.clone(), conflicted())));
    assert!(trashed.contains(&(library.sync_b.id.clone(), conflicted())));
}

#[test]
fn accepting_a_modified_document_records_its_new_content_in_the_catalog() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");
    library.folders.put(&library.sync_a, "letters/a.pdf", "changed");

    let catalog = done(library.keep_version().run(&document.path, &library.sync_a.id, true));

    assert_eq!(catalog.documents()[&document.id].content_hash, FakeFolders::hash_of("changed"));
    assert_eq!(library.folders.content(&library.sync_b, "letters/a.pdf").as_deref(), Some("changed"));
}
