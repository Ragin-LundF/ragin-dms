use std::collections::BTreeSet;

use dms_application::common::OperationOutcome;
use dms_domain::library::RelativePath;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::path;

fn energy() -> RelativePath {
    path("home/energy")
}

#[test]
fn a_confirmed_folder_goes_to_the_trash_in_every_folder_and_its_documents_leave_the_catalog() {
    let library = TestLibrary::new();
    library.open();
    library.add("home/energy/power.pdf", "power");
    library.add("home/energy/old/gas.pdf", "gas");
    library.add("home/water.pdf", "water");

    let outcome = library.trash_folder().run(&energy(), true);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    for folder in library.all_folders() {
        assert_eq!(library.folders.paths(&folder), BTreeSet::from(["home/water.pdf".to_owned()]));
        assert!(library.folders.trashed().contains(&(folder.id.clone(), energy())));
    }
    let left: Vec<String> =
        library.catalog().documents().values().map(|document| document.path.value().to_owned()).collect();
    assert_eq!(left, vec!["home/water.pdf".to_owned()]);
}

#[test]
fn nothing_happens_without_confirmation_and_the_top_level_cannot_be_deleted() {
    let library = TestLibrary::new();
    library.open();
    library.add("home/energy/power.pdf", "power");

    let unconfirmed = library.trash_folder().run(&energy(), false);
    let top_level = library.trash_folder().run(&RelativePath::root(), true);

    assert_eq!(unconfirmed, OperationOutcome::NotConfirmed);
    assert_eq!(top_level, OperationOutcome::NotFound);
    assert_eq!(library.catalog().documents().len(), 1);
    assert_eq!(library.folders.trashed(), Vec::new());
}

#[test]
fn an_offline_sync_folder_gets_the_trashing_from_the_journal() {
    let library = TestLibrary::new();
    library.open();
    library.add("home/energy/power.pdf", "power");
    library.folders.make_unreachable(&library.sync_b);

    let outcome = library.trash_folder().run(&energy(), true);

    let OperationOutcome::Done { pending_folders, .. } = outcome else { panic!("expected Done, got {outcome:?}") };
    assert_eq!(pending_folders, IndexSet::from([library.sync_b.id.clone()]));
    assert!(library.catalog().documents().is_empty());
}
