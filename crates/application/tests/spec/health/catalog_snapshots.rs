use dms_application::common::OperationOutcome;
use dms_application::health::SnapshotComparison;
use dms_domain::health::PartDiff;

use crate::support::TestLibrary;
use crate::support::test_data::done;

#[test]
fn every_saved_catalog_leaves_a_snapshot() {
    let library = TestLibrary::new();
    library.open();
    library.add("a.txt", "one");
    library.add("b.txt", "two");

    let revisions: Vec<u64> =
        library.snapshots.stored.lock().iter().map(|(snapshot, _)| snapshot.revision.value()).collect();
    let current = library.catalog().revision().value();
    assert_eq!(revisions.last(), Some(&current));
    assert!(revisions.len() >= 2);
}

#[test]
fn restoring_a_snapshot_raises_the_revision_and_reverts_the_catalog() {
    let library = TestLibrary::new();
    library.open();
    library.add("a.txt", "one");
    let before = library.catalog();
    let snapshot = library.snapshots.ids().pop().expect("a snapshot of the catalog with a.txt");
    library.add("b.txt", "two");

    let SnapshotComparison::Compared(diff) = library.compare_catalog_snapshot().run(&snapshot) else {
        panic!("expected a comparison");
    };
    assert_eq!(diff.documents, PartDiff { added: 0, removed: 1, changed: 0 });

    let revision_before = library.catalog().revision();
    let restored = done(library.restore_catalog_snapshot().run(&snapshot, true));

    assert_eq!(restored.revision(), revision_before.next());
    assert_eq!(restored.documents(), before.documents());
    assert_eq!(
        library.folders.catalog_in(&library.sync_b).map(|catalog| catalog.revision()),
        Some(restored.revision())
    );
    assert!(library.folders.content(&library.primary, "b.txt").is_some(), "files are never touched");
}

#[test]
fn nothing_is_restored_without_confirmation() {
    let library = TestLibrary::new();
    library.open();
    library.add("a.txt", "one");
    let snapshot = library.snapshots.ids().pop().expect("a snapshot");
    let revision = library.catalog().revision();

    assert_eq!(library.restore_catalog_snapshot().run(&snapshot, false), OperationOutcome::NotConfirmed);
    assert_eq!(library.catalog().revision(), revision);
}

#[test]
fn an_unknown_snapshot_fails_without_a_change() {
    let library = TestLibrary::new();
    library.open();
    let unknown = dms_application::port::health::CatalogSnapshotId::new("20200101T000000Z-r1").expect("id");

    assert!(matches!(library.restore_catalog_snapshot().run(&unknown, true), OperationOutcome::StorageFailed(_)));
    assert!(matches!(library.compare_catalog_snapshot().run(&unknown), SnapshotComparison::Failed(_)));
}

#[test]
fn snapshot_ids_never_leave_the_snapshot_folder() {
    use dms_application::port::health::CatalogSnapshotId;
    assert!(CatalogSnapshotId::new("../dms-catalog").is_err());
    assert!(CatalogSnapshotId::new("a/b").is_err());
    assert!(CatalogSnapshotId::new("").is_err());
    assert!(CatalogSnapshotId::new("20261111T120000Z-r42").is_ok());
}
