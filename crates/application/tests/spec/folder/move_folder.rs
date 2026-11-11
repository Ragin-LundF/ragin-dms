use std::collections::BTreeSet;

use dms_application::common::OperationOutcome;
use dms_domain::catalog::CatalogRejection;

use crate::support::TestLibrary;
use crate::support::test_data::path;

#[test]
fn a_folder_moves_with_its_documents_in_every_folder_and_in_the_catalog() {
    let library = TestLibrary::new();
    library.open();
    library.add("letters/a.pdf", "alpha");
    library.add("letters/b.pdf", "beta");

    let outcome = library.move_folder().run(&path("letters"), &path("archive/letters"));

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    let expected = BTreeSet::from(["archive/letters/a.pdf".to_owned(), "archive/letters/b.pdf".to_owned()]);
    for folder in library.all_folders() {
        assert_eq!(library.folders.paths(&folder), expected);
    }
    let catalogued: BTreeSet<String> =
        library.catalog().documents().values().map(|document| document.path.value().to_owned()).collect();
    assert_eq!(catalogued, expected);
}

#[test]
fn a_folder_cannot_move_onto_an_existing_folder() {
    let library = TestLibrary::new();
    library.open();
    library.add("letters/a.pdf", "alpha");
    library.add("archive/b.pdf", "beta");

    let outcome = library.move_folder().run(&path("letters"), &path("Archive"));

    assert_eq!(outcome, OperationOutcome::NameTaken);
}

#[test]
fn a_folder_cannot_move_into_itself() {
    let library = TestLibrary::new();
    library.open();
    library.add("letters/a.pdf", "alpha");

    let outcome = library.move_folder().run(&path("letters"), &path("letters/inner"));

    assert_eq!(outcome, OperationOutcome::Rejected(CatalogRejection::InvalidFolderMove));
}
