use std::collections::BTreeSet;

use dms_application::common::OperationOutcome;
use dms_domain::library::FileNameViolation;

use crate::support::TestLibrary;
use crate::support::test_data::path;

#[test]
fn a_document_is_moved_in_every_folder_and_in_the_catalog() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");
    let target = path("archive/2024/a.pdf");

    let outcome = library.move_document().run(&document.id, &target);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    for folder in library.all_folders() {
        assert_eq!(library.folders.paths(&folder), BTreeSet::from([target.value().to_owned()]));
    }
    assert_eq!(library.catalog().documents()[&document.id].path, target);
}

#[test]
fn a_rename_that_only_changes_the_case_is_allowed() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");

    let outcome = library.move_document().run(&document.id, &path("letters/A.pdf"));

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    assert_eq!(library.folders.paths(&library.sync_b), BTreeSet::from(["letters/A.pdf".to_owned()]));
}

#[test]
fn a_name_another_file_has_ignoring_case_is_refused() {
    let library = TestLibrary::new();
    library.open();
    library.add("letters/a.pdf", "alpha");
    let other = library.add("letters/b.pdf", "beta");

    let outcome = library.move_document().run(&other.id, &path("letters/A.PDF"));

    assert_eq!(outcome, OperationOutcome::NameTaken);
    assert_eq!(library.folders.content(&library.primary, "letters/b.pdf").as_deref(), Some("beta"));
}

#[test]
fn a_name_the_file_name_policy_refuses_is_not_used() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");

    let outcome = library.move_document().run(&document.id, &path("letters/a?.pdf"));

    assert_eq!(outcome, OperationOutcome::InvalidName(FileNameViolation::ForbiddenCharacter));
}
