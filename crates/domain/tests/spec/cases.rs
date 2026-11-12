//! Cases: how the user's changes move a case, and which dashboard tasks a case asks for.

use dms_domain::cases::{Case, CasePlanner, CaseStatus, EntryDirection, FollowUp, FollowUpId, FollowUpSource};
use dms_domain::task::OpenTask;

use crate::test_data::{self, Applied, catalog_with, date, document, document_id};

fn follow_up(id: &str, on: chrono::NaiveDate) -> FollowUp {
    FollowUp {
        id: FollowUpId::new(id).expect("id"),
        date: on,
        text: "Send the receipts".to_owned(),
        done: false,
        source: FollowUpSource::Manual,
    }
}

fn waiting_since(since: chrono::NaiveDate) -> Case {
    test_data::case("case-1", "Objection").with_status(CaseStatus::Waiting, since)
}

#[test]
fn an_answer_reopens_a_waiting_case_but_a_sent_letter_keeps_it_waiting() {
    let waiting = waiting_since(date(2026, 10, 1));

    let sent = waiting.with_entry(&document_id("reminder"), EntryDirection::Sent, date(2026, 10, 20));
    let answered = sent.with_entry(&document_id("answer"), EntryDirection::Received, date(2026, 10, 25));

    assert_eq!(sent.status, CaseStatus::Waiting);
    assert_eq!(sent.waiting_since, Some(date(2026, 10, 1)));
    assert_eq!(answered.status, CaseStatus::Open);
    assert_eq!(answered.waiting_since, None);
    assert_eq!(answered.entries.len(), 2);
}

#[test]
fn adding_a_document_twice_keeps_one_entry_with_the_new_direction() {
    let case = test_data::case("case-1", "Objection")
        .with_entry(&document_id("doc-1"), EntryDirection::Received, date(2026, 10, 1))
        .with_entry(&document_id("doc-1"), EntryDirection::Sent, date(2026, 10, 2));

    assert_eq!(case.entries.len(), 1);
    assert_eq!(case.entries[0].direction, EntryDirection::Sent);
    assert_eq!(case.without_document(&document_id("doc-1")).entries, Vec::new());
}

#[test]
fn the_status_records_since_when_a_case_waits_and_when_it_was_closed() {
    let waiting = waiting_since(date(2026, 10, 1));
    let still_waiting = waiting.with_status(CaseStatus::Waiting, date(2026, 10, 9));
    let done = still_waiting.with_status(CaseStatus::Done, date(2026, 10, 10));
    let reopened = done.with_status(CaseStatus::Open, date(2026, 10, 11));

    assert_eq!(still_waiting.waiting_since, Some(date(2026, 10, 1)));
    assert_eq!((done.closed_on, done.waiting_since), (Some(date(2026, 10, 10)), None));
    assert_eq!((reopened.status, reopened.closed_on, reopened.waiting_since), (CaseStatus::Open, None, None));
}

#[test]
fn snoozing_moves_a_follow_up_from_its_date_or_from_today_when_it_has_passed() {
    let case = test_data::case("case-1", "Objection")
        .with_follow_up(follow_up("later", date(2026, 11, 30)))
        .with_follow_up(follow_up("overdue", date(2026, 11, 1)));
    let today = date(2026, 11, 11);

    let later = case.snoozing(&FollowUpId::new("later").expect("id"), 7, today).expect("follow-up");
    let overdue = case.snoozing(&FollowUpId::new("overdue").expect("id"), 7, today).expect("follow-up");

    assert_eq!(later.follow_ups[0].date, date(2026, 12, 7));
    assert_eq!(overdue.follow_ups[1].date, date(2026, 11, 18));
    assert!(case.snoozing(&FollowUpId::new("missing").expect("id"), 7, today).is_none());
    assert!(case.completing(&FollowUpId::new("missing").expect("id")).is_none());
}

#[test]
fn a_follow_up_is_due_from_its_date_until_it_is_done() {
    let case = test_data::case("case-1", "Tax office asks for receipts")
        .with_follow_up(follow_up("f-1", date(2026, 11, 30)))
        .with_follow_up(follow_up("f-2", date(2026, 11, 30)))
        .completing(&FollowUpId::new("f-2").expect("id"))
        .expect("follow-up");
    let catalog = catalog_with(&[document("doc-1", "a.pdf")]).save_case(case.clone()).applied();

    assert_eq!(CasePlanner::due_tasks(&catalog, date(2026, 11, 29)), Vec::new());
    assert_eq!(
        CasePlanner::due_tasks(&catalog, date(2026, 11, 30)),
        vec![OpenTask::FollowUpDue {
            case_id: case.id.clone(),
            follow_up_id: FollowUpId::new("f-1").expect("id"),
            date: date(2026, 11, 30),
        }]
    );
    assert_eq!(CasePlanner::due_tasks(&catalog, date(2026, 12, 3)).len(), 1);
    let closed = catalog.save_case(case.with_status(CaseStatus::Done, date(2026, 11, 2))).applied();
    assert_eq!(CasePlanner::due_tasks(&closed, date(2026, 12, 3)), Vec::new());
}

#[test]
fn a_waiting_case_without_an_answer_after_its_reminder_days_asks_for_attention() {
    let case = waiting_since(date(2026, 10, 1));
    let catalog = catalog_with(&[]).save_case(case.clone()).applied();

    assert_eq!(CasePlanner::due_tasks(&catalog, date(2026, 10, 21)), Vec::new());
    assert_eq!(
        CasePlanner::due_tasks(&catalog, date(2026, 10, 22)),
        vec![OpenTask::CaseWaitingTooLong { case_id: case.id.clone(), since: date(2026, 10, 1) }]
    );
    let answered = case.with_entry(&document_id("doc-1"), EntryDirection::Received, date(2026, 10, 30));
    let catalog = catalog_with(&[document("doc-1", "a.pdf")]).save_case(answered).applied();
    assert_eq!(CasePlanner::due_tasks(&catalog, date(2026, 11, 30)), Vec::new());
}
