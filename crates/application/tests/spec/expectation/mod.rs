//! Tests of `dms_application::expectation` and the "expected but missing" tasks of the dashboard.

use chrono::{Days, NaiveDate};
use dms_application::port::system::Clock;
use dms_application::resolution::DismissTask;
use dms_domain::document::DocumentMetadata;
use dms_domain::task::OpenTask;

use crate::support::TestLibrary;

fn expected_tasks(library: &TestLibrary) -> Vec<OpenTask> {
    let dashboard = library.dashboard(None).expect("a dashboard");
    dashboard.tasks.into_iter().filter(|task| matches!(task, OpenTask::ExpectedDocumentMissing(_))).collect()
}

/// Monthly slips up to two months before the clock's day, so the next one is missing.
fn library_with_monthly_slips() -> TestLibrary {
    let library = TestLibrary::new();
    library.open();
    let last = library.clock.today() - Days::new(45);
    for month in 0..4u64 {
        let on: NaiveDate = last - Days::new(30 * month);
        let document = library.add(&format!("salary/{month}.pdf"), &format!("slip {month}"));
        let metadata =
            DocumentMetadata { correspondent: Some("Employer Ltd".to_owned()), document_date: Some(on), title: None };
        let _ = library.store.commit(|catalog| catalog.update_metadata(&document.id, metadata.clone()));
    }
    library
}

#[test]
fn requested_hides_the_task_for_fourteen_days_on_this_device() {
    let library = library_with_monthly_slips();
    let tasks = expected_tasks(&library);
    assert_eq!(tasks.len(), 1, "got {tasks:?}");

    library.snooze_expected_document().run(&tasks[0].key()).expect("snoozed");

    assert_eq!(expected_tasks(&library), Vec::new());
    let snoozes = library.tasks.snoozes.get();
    assert_eq!(snoozes.snoozed_until.get(&tasks[0].key()), Some(&(library.clock.today() + Days::new(14))));
}

#[test]
fn ignore_once_dismisses_only_this_period_s_task() {
    let library = library_with_monthly_slips();
    let task = expected_tasks(&library).remove(0);

    DismissTask::new(library.tasks.clone()).run(&task.key()).expect("dismissed");

    assert_eq!(expected_tasks(&library), Vec::new());
    assert!(task.key().value().ends_with(&dms_domain::iso_date(match &task {
        OpenTask::ExpectedDocumentMissing(expected) => expected.expected_on,
        _ => unreachable!("filtered"),
    })));
}
