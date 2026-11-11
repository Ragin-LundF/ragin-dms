use dms_application::common::OperationOutcome;
use dms_application::port::storage::DocumentStorage;
use dms_domain::change::DetectedChange;

use crate::support::TestLibrary;

#[test]
fn a_file_deleted_outside_the_tool_is_restored_from_a_folder_that_still_has_it() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");
    library.folders.move_to_trash(&library.primary, &document.path).expect("trashed");
    library.folders.move_to_trash(&library.sync_a, &document.path).expect("trashed");
    let missing = library
        .scan_changes()
        .into_iter()
        .find_map(|change| match change {
            DetectedChange::MissingFile(missing) => Some(missing),
            _ => None,
        })
        .expect("a missing file");

    let outcome = library.restore_document().run(&missing);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "letters/a.pdf").as_deref(), Some("alpha"));
    }
}
