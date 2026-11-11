use std::collections::HashSet;

use dms_application::common::OperationOutcome;
use dms_domain::change::{DetectedChange, NewFile};

use crate::support::TestLibrary;
use crate::support::test_data::path;

fn new_file_of(library: &TestLibrary) -> NewFile {
    let mut new_files: Vec<NewFile> = library
        .scan_changes()
        .into_iter()
        .filter_map(|change| match change {
            DetectedChange::NewFile(new_file) => Some(new_file),
            _ => None,
        })
        .collect();
    assert_eq!(new_files.len(), 1);
    new_files.remove(0)
}

#[test]
fn a_new_file_that_is_a_copy_of_a_document_goes_to_the_trash_in_every_folder_that_has_it() {
    let library = TestLibrary::new();
    library.open();
    library.add("letters/a.pdf", "alpha");
    library.folders.put(&library.primary, "inbox/a copy.pdf", "alpha");
    library.folders.put(&library.sync_a, "inbox/a copy.pdf", "alpha");
    let new_file = new_file_of(&library);

    let refused = library.trash_duplicate_file().run(&new_file, false);
    let outcome = library.trash_duplicate_file().run(&new_file, true);

    assert_eq!(refused, OperationOutcome::NotConfirmed);
    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    let copy = path("inbox/a copy.pdf");
    let expected = HashSet::from([(library.primary.id.clone(), copy.clone()), (library.sync_a.id.clone(), copy)]);
    assert_eq!(library.folders.trashed().into_iter().collect::<HashSet<_>>(), expected);
    assert_eq!(library.folders.content(&library.primary, "letters/a.pdf").as_deref(), Some("alpha"));
}

#[test]
fn a_file_that_only_looks_like_a_copy_stays_where_it_is() {
    let library = TestLibrary::new();
    library.open();
    library.add("letters/a.pdf", "alpha");
    library.folders.put(&library.sync_a, "letters/a (1).pdf", "alphb");

    let outcome = library.trash_duplicate_file().run(&new_file_of(&library), true);

    assert_eq!(outcome, OperationOutcome::NotFound);
    assert_eq!(library.folders.trashed(), Vec::new());
}
