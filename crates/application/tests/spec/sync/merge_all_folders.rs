use std::collections::BTreeSet;

use dms_application::port::storage::DocumentStorage;
use dms_application::sync::{SyncOutcome, SyncReport};

use crate::support::TestLibrary;
use crate::support::test_data::{Applied, path};

#[track_caller]
fn completed(outcome: SyncOutcome) -> SyncReport {
    match outcome {
        SyncOutcome::Completed(report) => report,
        other => panic!("expected Completed, got {other:?}"),
    }
}

#[test]
fn files_missing_anywhere_are_copied_conflicts_are_not_and_the_catalogs_are_merged() {
    let library = TestLibrary::new();
    let opened = library.open();
    library.folders.put(&library.sync_a, "inbox/x.pdf", "x");
    library.folders.put(&library.sync_b, "inbox/y.pdf", "y");
    library.folders.put(&library.primary, "c.txt", "one");
    library.folders.put(&library.sync_b, "c.txt", "two");
    let catalogued = TestLibrary::document("d-y", "inbox/y.pdf", "y");
    library.folders.set_catalog(&library.sync_b, opened.add_document(catalogued.clone()).applied());

    let report = completed(library.merge_all_folders().run());

    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "inbox/x.pdf").as_deref(), Some("x"));
        assert_eq!(library.folders.content(&folder, "inbox/y.pdf").as_deref(), Some("y"));
    }
    assert_eq!(library.folders.content(&library.sync_a, "c.txt"), None);
    assert_eq!(report.conflicts, vec![path("c.txt")]);
    assert!(library.catalog().documents().contains_key(&catalogued.id));
}

#[test]
fn a_document_moved_outside_the_tool_is_not_turned_into_a_duplicate() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");
    library.folders.move_entry(&library.sync_a, &document.path, &path("old/a.pdf")).expect("moved");

    let report = completed(library.merge_all_folders().run());

    assert_eq!(library.folders.paths(&library.sync_a), BTreeSet::from(["old/a.pdf".to_owned()]));
    assert_eq!(library.folders.paths(&library.primary), BTreeSet::from(["letters/a.pdf".to_owned()]));
    assert_eq!(report.left_alone.get(&library.sync_a.id), Some(&vec![path("old/a.pdf")]));
}
