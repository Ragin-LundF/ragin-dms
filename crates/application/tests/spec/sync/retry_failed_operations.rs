use std::collections::BTreeSet;

use dms_application::port::storage::{OperationJournal, PendingOperation};
use dms_application::sync::{RetryOutcome, RetryReport};

use crate::support::TestLibrary;
use crate::support::test_data::done;

#[test]
fn operations_for_a_folder_that_is_back_are_applied_in_order() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.sync_b);
    library.add("letters/a.pdf", "alpha");
    library.folders.make_reachable(&library.sync_b);

    let report = library.retried();

    assert_eq!(report, Some(RetryReport { succeeded: 2, ..RetryReport::default() }));
    assert_eq!(library.folders.content(&library.sync_b, "letters/a.pdf").as_deref(), Some("alpha"));
    assert_eq!(library.folders.catalog_in(&library.sync_b), Some(library.catalog()));
    assert_eq!(library.journal.pending().expect("readable"), Vec::<PendingOperation>::new());
}

#[test]
fn an_operation_whose_file_is_gone_is_dropped_and_the_rest_continue() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.sync_b);
    let document = library.add("letters/a.pdf", "alpha");
    done(library.trash_document().run(&document.id, true));
    library.folders.make_reachable(&library.sync_b);

    let report = library.retried();

    assert_eq!(report, Some(RetryReport { succeeded: 3, dropped: 1, still_pending: 0 }));
    assert_eq!(library.folders.paths(&library.sync_b), BTreeSet::new());
    assert_eq!(library.journal.pending().expect("readable"), Vec::<PendingOperation>::new());
}

#[test]
fn operations_for_a_folder_that_is_still_offline_stay_in_the_journal() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.sync_b);
    library.add("letters/a.pdf", "alpha");

    let report = library.retried();

    assert_eq!(report, Some(RetryReport { still_pending: 2, ..RetryReport::default() }));
    assert_eq!(library.journal.pending().expect("readable").len(), 2);
}

#[test]
fn nothing_is_retried_while_no_library_is_open() {
    assert_eq!(TestLibrary::new().retry_failed_operations().run(), RetryOutcome::LibraryNotOpen);
}
