use dms_application::common::OperationOutcome;

use crate::support::TestLibrary;
use crate::support::test_data::path;

#[test]
fn a_folder_is_created_in_every_library_folder() {
    let library = TestLibrary::new();
    library.open();

    let outcome = library.create_folder().run(&path("archive"));

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    for folder in library.all_folders() {
        assert!(library.folders.has_directory(&folder, "archive"));
    }
}

#[test]
fn a_folder_name_that_differs_only_by_case_from_an_existing_one_is_refused() {
    let library = TestLibrary::new();
    library.open();
    library.create_folder().run(&path("archive"));

    let outcome = library.create_folder().run(&path("Archive"));

    assert_eq!(outcome, OperationOutcome::NameTaken);
}
