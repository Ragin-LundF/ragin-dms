use dms_application::common::{NoProgress, OperationOutcome};
use dms_application::health::VerifyOutcome;
use dms_domain::health::HealthFinding;

use crate::support::TestLibrary;
use crate::support::test_data::path;

fn damaged_in_sync_a(library: &TestLibrary) -> dms_domain::document::Document {
    library.open();
    let document = library.add("letters/a.txt", "original");
    library.folders.put(&library.sync_a, "letters/a.txt", "origiNAL");
    assert!(matches!(library.verify_library().run(&NoProgress), VerifyOutcome::Finished(_)));
    document
}

#[test]
fn nothing_is_repaired_without_confirmation() {
    let library = TestLibrary::new();
    let document = damaged_in_sync_a(&library);

    let outcome = library.repair_copy().run(&document.id, &library.sync_a.id, &library.primary.id, false);

    assert_eq!(outcome, OperationOutcome::NotConfirmed);
    assert_eq!(library.folders.content(&library.sync_a, "letters/a.txt").as_deref(), Some("origiNAL"));
}

#[test]
fn the_damaged_copy_goes_to_the_trash_and_the_intact_one_takes_its_place() {
    let library = TestLibrary::new();
    let document = damaged_in_sync_a(&library);

    let outcome = library.repair_copy().run(&document.id, &library.sync_a.id, &library.primary.id, true);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    assert_eq!(library.folders.content(&library.sync_a, "letters/a.txt").as_deref(), Some("original"));
    assert_eq!(library.folders.trashed(), vec![(library.sync_a.id.clone(), path("letters/a.txt"))]);
    let report = library.health.report.lock().clone().expect("a report");
    assert_eq!(report.findings, Vec::<HealthFinding>::new());
}

#[test]
fn a_damaged_primary_is_repaired_from_a_sync_folder() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("a.txt", "original");
    library.folders.put(&library.primary, "a.txt", "originaX");

    let outcome = library.repair_copy().run(&document.id, &library.primary.id, &library.sync_b.id, true);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    assert_eq!(library.folders.content(&library.primary, "a.txt").as_deref(), Some("original"));
    assert_eq!(library.folders.trashed(), vec![(library.primary.id.clone(), path("a.txt"))]);
}

#[test]
fn a_source_that_no_longer_matches_the_catalog_never_spreads() {
    let library = TestLibrary::new();
    let document = damaged_in_sync_a(&library);
    library.folders.put(&library.primary, "letters/a.txt", "changed!");

    let outcome = library.repair_copy().run(&document.id, &library.sync_a.id, &library.primary.id, true);

    assert_eq!(outcome, OperationOutcome::SourceChanged);
    assert_eq!(library.folders.content(&library.sync_a, "letters/a.txt").as_deref(), Some("origiNAL"));
    assert_eq!(library.folders.trashed(), Vec::new());
}

#[test]
fn a_repair_into_an_offline_folder_waits_in_the_journal() {
    let library = TestLibrary::new();
    let document = damaged_in_sync_a(&library);
    library.folders.make_unreachable(&library.sync_a);

    let outcome = library.repair_copy().run(&document.id, &library.sync_a.id, &library.primary.id, true);

    let OperationOutcome::Done { pending_folders, .. } = outcome else { panic!("expected Done, got {outcome:?}") };
    assert!(pending_folders.contains(&library.sync_a.id));
    assert_eq!(library.journal.operations.lock().len(), 1);
}
