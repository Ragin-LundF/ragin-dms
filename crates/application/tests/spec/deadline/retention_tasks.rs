//! `RetentionTasksTest`.

use chrono::Days;
use dms_application::common::OperationOutcome;
use dms_application::deadline::DeadlineCalendar;
use dms_application::document::DetailEdit;
use dms_application::port::system::Clock;
use dms_domain::field::{FieldPresets, FieldValue};
use dms_domain::retention::Retention;
use dms_domain::settings::CatalogSettings;
use dms_domain::task::OpenTask;
use indexmap::IndexMap;

use crate::support::TestLibrary;

fn retention_tasks(library: &TestLibrary) -> Vec<OpenTask> {
    let dashboard = library.dashboard(None).expect("a dashboard");
    dashboard.tasks.into_iter().filter(|task| matches!(task, OpenTask::RetentionDue { .. })).collect()
}

fn keep_until(library: &TestLibrary, document_id: &dms_domain::document::DocumentId, date: chrono::NaiveDate) {
    library.edit_document_details().run(&DetailEdit::SetRetention {
        document_id: document_id.clone(),
        retention: Some(Retention { keep_until: Some(date), ..Retention::default() }),
    });
}

#[test]
fn no_document_is_ever_flagged_unless_the_user_set_a_retention() {
    let library = TestLibrary::new();
    library.open();
    library.add("old.pdf", "a");

    assert_eq!(retention_tasks(&library), Vec::new());
}

#[test]
fn an_ended_retention_is_a_task_until_it_is_marked_as_done_and_then_never_again() {
    let library = TestLibrary::new();
    library.open();
    let first = library.add("first.pdf", "a");
    let second = library.add("second.pdf", "b");
    for document in [&first, &second] {
        keep_until(&library, &document.id, library.clock.today());
    }

    let before = retention_tasks(&library);
    let settled = library.settle_retention().run(&[first.id.clone(), second.id.clone()]);
    let after = retention_tasks(&library);

    assert_eq!(before.len(), 2);
    assert!(matches!(settled, OperationOutcome::Done { .. }), "got {settled:?}");
    assert_eq!(after, Vec::new());
    let retention = library.catalog().documents()[&first.id].retention.clone().expect("a retention");
    assert_eq!(retention.settled_at, Some(library.clock.now()));
    assert!(retention.keep_until.is_some());
}

#[test]
fn deadlines_within_their_lead_time_are_tasks_and_the_next_dates_are_upcoming() {
    let library = TestLibrary::new();
    library.open();
    library
        .change_settings()
        .run("fields", |_| CatalogSettings { field_definitions: FieldPresets::all(), ..CatalogSettings::default() });
    let bill = library.add("bill.pdf", "a");
    let due = library.clock.today().checked_add_days(Days::new(5)).expect("a date");
    library.edit_document_details().run(&DetailEdit::SetFields {
        document_id: bill.id.clone(),
        fields: IndexMap::from([(FieldPresets::due_date().key, FieldValue::Date(due))]),
    });

    let dashboard = library.dashboard(None).expect("a dashboard");

    let deadlines: Vec<OpenTask> =
        dashboard.tasks.into_iter().filter(|task| matches!(task, OpenTask::DeadlineDue { .. })).collect();
    assert_eq!(
        deadlines,
        vec![OpenTask::DeadlineDue { document_id: bill.id, field_key: FieldPresets::due_date().key, date: due }]
    );
    assert_eq!(dashboard.upcoming.iter().map(|upcoming| upcoming.date).collect::<Vec<_>>(), vec![due]);
}

#[test]
fn the_calendar_export_has_one_all_day_event_per_date_with_escaped_text() {
    let library = TestLibrary::new();
    library.open();
    let bill = library.add("bill; final, v2.pdf", "a");
    keep_until(&library, &bill.id, library.clock.today().checked_add_days(Days::new(3)).expect("a date"));

    let calendar = DeadlineCalendar::of(&library.catalog(), library.clock.today());

    assert!(calendar.starts_with("BEGIN:VCALENDAR\r\n"));
    assert!(calendar.contains("DTSTART;VALUE=DATE:20240304\r\n"));
    assert!(calendar.contains("SUMMARY:Retention ends: bill\\; final\\, v2.pdf\r\n"));
    assert_eq!(calendar.matches("BEGIN:VEVENT").count(), 1);
}
