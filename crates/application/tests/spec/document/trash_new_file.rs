use dms_application::common::OperationOutcome;
use dms_domain::change::DetectedChange;

use crate::support::TestLibrary;
use crate::support::test_data::path;

#[test]
fn a_file_found_on_one_side_only_goes_to_the_trash_there_and_only_when_confirmed() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.sync_a, "inbox/stray.pdf", "stray");
    let new_file = library
        .scan_changes()
        .into_iter()
        .find_map(|change| match change {
            DetectedChange::NewFile(new_file) => Some(new_file),
            _ => None,
        })
        .expect("a new file");

    let refused = library.trash_new_file().run(&new_file, false);
    assert_eq!(library.folders.trashed(), Vec::new());
    let outcome = library.trash_new_file().run(&new_file, true);

    assert_eq!(refused, OperationOutcome::NotConfirmed);
    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    assert_eq!(library.folders.trashed(), vec![(library.sync_a.id.clone(), path("inbox/stray.pdf"))]);
}
