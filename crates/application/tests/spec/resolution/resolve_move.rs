use std::collections::BTreeSet;

use dms_application::common::OperationOutcome;
use dms_application::port::storage::DocumentStorage;
use dms_application::resolution::MoveDecision;
use dms_domain::change::{DetectedChange, MovedFile};
use dms_domain::library::RelativePath;

use crate::support::TestLibrary;
use crate::support::test_data::path;

fn moved() -> RelativePath {
    path("archive/a.pdf")
}

fn moved_in_sync_folder(library: &TestLibrary) -> MovedFile {
    library.open();
    let document = library.add("letters/a.pdf", "alpha");
    library.folders.move_entry(&library.sync_a, &document.path, &moved()).expect("moved");
    library
        .scan_changes()
        .into_iter()
        .find_map(|change| match change {
            DetectedChange::MovedFile(moved) => Some(moved),
            _ => None,
        })
        .expect("a moved file")
}

#[test]
fn an_applied_move_reaches_every_folder_and_the_catalog() {
    let library = TestLibrary::new();
    let change = moved_in_sync_folder(&library);

    let outcome = library.resolve_move().run(&change, MoveDecision::Apply);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    for folder in library.all_folders() {
        assert_eq!(library.folders.paths(&folder), BTreeSet::from([moved().value().to_owned()]));
    }
    assert_eq!(library.catalog().documents()[&change.document_id].path, moved());
}

#[test]
fn an_undone_move_puts_the_file_back_where_the_catalog_has_it() {
    let library = TestLibrary::new();
    let change = moved_in_sync_folder(&library);

    let outcome = library.resolve_move().run(&change, MoveDecision::Undo);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    assert_eq!(library.folders.paths(&library.sync_a), BTreeSet::from(["letters/a.pdf".to_owned()]));
    assert_eq!(library.catalog().documents()[&change.document_id].path, change.previous_path);
}
