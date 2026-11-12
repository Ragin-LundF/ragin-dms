use dms_application::cases::{NewCase, NewFollowUp};
use dms_application::common::OperationOutcome;
use dms_application::port::system::Clock;
use dms_domain::cases::{Case, CaseId, CaseStatus, EntryDirection, FollowUpId, FollowUpSource};
use dms_domain::catalog::CatalogRejection;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::task::OpenTask;

use crate::support::TestLibrary;
use crate::support::test_data::{date, document_id, done};

fn library_with_letter() -> (TestLibrary, Document) {
    let library = TestLibrary::new();
    library.open();
    let letter = library.add("tax/request.pdf", "please send the receipts");
    let metadata = DocumentMetadata {
        title: None,
        document_date: Some(date(2024, 2, 20)),
        correspondent: Some("Tax office".to_owned()),
    };
    done(library.store.commit(|catalog| catalog.update_metadata(&letter.id, metadata.clone())));
    (library, letter)
}

fn follow_up(text: &str, on: chrono::NaiveDate, source: FollowUpSource) -> NewFollowUp {
    NewFollowUp { text: text.to_owned(), date: on, source }
}

fn case_of(library: &TestLibrary, id: &CaseId) -> Case {
    library.catalog().cases().get(id).cloned().expect("the case")
}

fn case_tasks(library: &TestLibrary) -> Vec<OpenTask> {
    let dashboard = library.dashboard(None).expect("a dashboard");
    let is_case_task =
        |task: &OpenTask| matches!(task, OpenTask::FollowUpDue { .. } | OpenTask::CaseWaitingTooLong { .. });
    dashboard.tasks.into_iter().filter(is_case_task).collect()
}

#[test]
fn an_accepted_request_opens_a_case_whose_follow_up_shows_on_the_dashboard_on_its_day() {
    let (library, letter) = library_with_letter();
    let today = library.clock.today();
    let source = FollowUpSource::Ai { document_id: letter.id.clone() };

    let created = library.cases().create_case.run(&NewCase {
        title: "  Tax office asks for receipts ".to_owned(),
        correspondent: Some(" Tax office ".to_owned()),
        documents: vec![(letter.id.clone(), EntryDirection::Received)],
        follow_ups: vec![
            follow_up("Send the receipts", today, source.clone()),
            follow_up("Check the answer", date(2024, 3, 30), FollowUpSource::Manual),
        ],
    });

    assert!(matches!(created.outcome, OperationOutcome::Done { .. }), "{:?}", created.outcome);
    let case = case_of(&library, &created.case_id);
    assert_eq!(
        (case.title.as_str(), case.correspondent.as_deref()),
        ("Tax office asks for receipts", Some("Tax office"))
    );
    assert_eq!((case.status, case.opened_on), (CaseStatus::Open, today));
    assert_eq!(case.entries[0].date, date(2024, 2, 20), "an entry is dated by its document date");
    assert_eq!(case.follow_ups[0].source, source);
    assert_eq!(
        case_tasks(&library),
        vec![OpenTask::FollowUpDue {
            case_id: created.case_id,
            follow_up_id: case.follow_ups[0].id.clone(),
            date: today
        }]
    );
}

#[test]
fn a_case_is_refused_without_a_title_or_with_a_document_that_is_gone() {
    let (library, letter) = library_with_letter();
    let new =
        NewCase { title: "Objection".to_owned(), correspondent: None, documents: Vec::new(), follow_ups: Vec::new() };

    let blank = library.cases().create_case.run(&NewCase { title: " ".to_owned(), ..new.clone() });
    let gone = library.cases().create_case.run(&NewCase {
        documents: vec![(letter.id, EntryDirection::Received), (document_id("gone"), EntryDirection::Sent)],
        ..new
    });

    assert!(matches!(blank.outcome, OperationOutcome::InvalidInput(_)), "{:?}", blank.outcome);
    assert_eq!(gone.outcome, OperationOutcome::Rejected(CatalogRejection::DocumentNotFound));
    assert!(library.catalog().cases().is_empty());
}

#[test]
fn an_answer_to_a_waiting_case_opens_it_again_and_ends_the_reminder() {
    let (library, letter) = library_with_letter();
    let services = library.cases();
    let created = services.create_case.run(&NewCase {
        title: "Objection".to_owned(),
        correspondent: None,
        documents: Vec::new(),
        follow_ups: Vec::new(),
    });
    done(services.set_case_status.run(&created.case_id, CaseStatus::Waiting));
    let long_ago = case_of(&library, &created.case_id);
    let since = date(2024, 2, 9);
    done(library.store.commit(|catalog| catalog.save_case(Case { waiting_since: Some(since), ..long_ago.clone() })));
    assert_eq!(case_tasks(&library), vec![OpenTask::CaseWaitingTooLong { case_id: created.case_id.clone(), since }]);

    done(services.add_to_case.run(&created.case_id, &letter.id, EntryDirection::Received));

    let case = case_of(&library, &created.case_id);
    assert_eq!((case.status, case.waiting_since), (CaseStatus::Open, None));
    assert_eq!(case_tasks(&library), Vec::new());
    done(services.remove_from_case.run(&created.case_id, &letter.id));
    assert_eq!(case_of(&library, &created.case_id).entries, Vec::new());
    assert!(library.catalog().documents().contains_key(&letter.id), "the document stays in the library");
}

#[test]
fn follow_ups_are_completed_or_snoozed_and_a_lost_one_is_refused() {
    let (library, _) = library_with_letter();
    let services = library.cases();
    let today = library.clock.today();
    let created = services.create_case.run(&NewCase {
        title: "Claim".to_owned(),
        correspondent: None,
        documents: Vec::new(),
        follow_ups: vec![follow_up("Call the expert", today, FollowUpSource::Manual)],
    });
    let follow_up_id = case_of(&library, &created.case_id).follow_ups[0].id.clone();

    done(services.snooze_follow_up.run(&created.case_id, &follow_up_id, 7));
    assert_eq!(case_of(&library, &created.case_id).follow_ups[0].date, date(2024, 3, 8));
    assert_eq!(case_tasks(&library), Vec::new());
    done(services.complete_follow_up.run(&created.case_id, &follow_up_id));
    assert!(case_of(&library, &created.case_id).follow_ups[0].done);
    done(services.add_case_note.run(&created.case_id, " Called them on Monday "));
    assert_eq!(case_of(&library, &created.case_id).notes[0].text, "Called them on Monday");

    let lost = services.complete_follow_up.run(&created.case_id, &FollowUpId::new("lost").expect("id"));
    let unknown = services.set_case_status.run(&CaseId::new("unknown").expect("id"), CaseStatus::Done);
    let blank_note = services.add_case_note.run(&created.case_id, " ");
    assert!(matches!(lost, OperationOutcome::InvalidInput(_)), "{lost:?}");
    assert_eq!(unknown, OperationOutcome::Rejected(CatalogRejection::CaseNotFound));
    assert!(matches!(blank_note, OperationOutcome::InvalidInput(_)), "{blank_note:?}");
}

#[test]
fn a_case_is_deleted_only_when_confirmed_and_its_documents_stay() {
    let (library, letter) = library_with_letter();
    let services = library.cases();
    let created = services.create_case.run(&NewCase {
        title: "Objection".to_owned(),
        correspondent: None,
        documents: vec![(letter.id.clone(), EntryDirection::Received)],
        follow_ups: Vec::new(),
    });

    assert_eq!(services.delete_case.run(&created.case_id, false), OperationOutcome::NotConfirmed);
    assert_eq!(library.catalog().cases().len(), 1);
    done(services.delete_case.run(&created.case_id, true));
    assert!(library.catalog().cases().is_empty());
    assert!(library.catalog().documents().contains_key(&letter.id));
}
