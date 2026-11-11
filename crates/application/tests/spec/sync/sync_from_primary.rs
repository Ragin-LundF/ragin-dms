use std::collections::BTreeSet;

use dms_application::sync::{SyncAction, SyncActionKind, SyncOutcome, SyncReport};
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::{done, path};

#[track_caller]
fn completed(outcome: SyncOutcome) -> SyncReport {
    match outcome {
        SyncOutcome::Completed(report) => report,
        other => panic!("expected Completed, got {other:?}"),
    }
}

fn action(library: &TestLibrary, kind: SyncActionKind, raw_path: &str) -> SyncAction {
    SyncAction { folder_id: library.sync_a.id.clone(), kind, path: path(raw_path) }
}

#[test]
fn a_sync_folder_gets_what_it_missed_keeps_its_own_files_and_nothing_is_overwritten() {
    let library = TestLibrary::new();
    library.open();
    let moved = library.add("letters/b.pdf", "beta");
    library.folders.make_unreachable(&library.sync_a);
    library.add("letters/a.pdf", "alpha");
    done(library.move_document().run(&moved.id, &path("archive/b.pdf")));
    library.folders.make_reachable(&library.sync_a);
    library.folders.put(&library.sync_a, "only-here.txt", "own");
    library.folders.put(&library.primary, "notes/n.txt", "one");
    library.folders.put(&library.sync_a, "notes/n.txt", "two");

    let report = completed(library.sync_from_primary().run());

    assert!(report.actions.contains(&action(&library, SyncActionKind::Moved, "archive/b.pdf")));
    assert!(report.actions.contains(&action(&library, SyncActionKind::Copied, "letters/a.pdf")));
    assert_eq!(report.conflicts, vec![path("notes/n.txt")]);
    assert_eq!(report.left_alone.get(&library.sync_a.id), Some(&vec![path("only-here.txt")]));
    let expected: BTreeSet<String> =
        ["archive/b.pdf", "letters/a.pdf", "notes/n.txt", "only-here.txt"].map(str::to_owned).into();
    assert_eq!(library.folders.paths(&library.sync_a), expected);
    assert_eq!(library.folders.content(&library.sync_a, "notes/n.txt").as_deref(), Some("two"));
    assert_eq!(library.folders.catalog_in(&library.sync_a), Some(library.catalog()));
}

#[test]
fn an_unreachable_sync_folder_is_reported() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.sync_b);

    let report = completed(library.sync_from_primary().run());

    assert_eq!(report.unreachable_folders, IndexSet::from([library.sync_b.id.clone()]));
}
