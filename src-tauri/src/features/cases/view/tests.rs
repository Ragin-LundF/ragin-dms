//! The cases presenters on synthetic catalogs; invented names only.

use chrono::{NaiveDate, TimeZone, Utc};
use dms_domain::cases::{Case, CaseId, CaseStatus, EntryDirection, FollowUp, FollowUpId, FollowUpSource};
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::classification::{ClassificationSuggestion, RequestedAction};
use dms_domain::document::{ContentHash, Document, DocumentId, DocumentMetadata};
use dms_domain::library::RelativePath;
use dms_domain::link::DocumentLink;

use super::*;
use crate::features::cases::dto::{CaseStatusDto, CaseTimelineKindDto, CasesFilterDto};
use crate::features::dashboard::dto::DashboardAction;

fn day(year: i32, month: u32, date: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, date).expect("date")
}

fn today() -> NaiveDate {
    day(2026, 11, 11)
}

fn id(value: &str) -> DocumentId {
    DocumentId::new(value).expect("id")
}

fn document(index: u32, title: &str, correspondent: &str) -> Document {
    let added_at = Utc.with_ymd_and_hms(2026, 1, index, 9, 0, 0).single().expect("instant");
    let document = Document::new(
        id(&format!("d{index}")),
        RelativePath::of(&format!("letters/{index}.pdf")).expect("path"),
        ContentHash::new(format!("{index:064x}")).expect("hash"),
        1_000,
        added_at,
    )
    .expect("document");
    Document {
        metadata: DocumentMetadata {
            title: Some(title.to_owned()),
            document_date: Some(day(2026, 11, index)),
            correspondent: Some(correspondent.to_owned()),
        },
        ..document
    }
}

fn follow_up(id: &str, on: NaiveDate, source: FollowUpSource) -> FollowUp {
    FollowUp { id: FollowUpId::new(id).expect("id"), date: on, text: format!("Do {id}"), done: false, source }
}

fn case(id: &str, title: &str, correspondent: &str) -> Case {
    let mut case = Case::new(CaseId::new(id).expect("id"), title, day(2026, 10, 1)).expect("case");
    case.correspondent = Some(correspondent.to_owned());
    case
}

fn applied(update: CatalogUpdate) -> Catalog {
    match update {
        CatalogUpdate::Applied(catalog) => catalog,
        CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
    }
}

fn catalog(documents: &[Document], cases: &[Case]) -> Catalog {
    let with_documents =
        documents.iter().fold(Catalog::empty(LibraryId::new("lib").expect("id")), |catalog, document| {
            applied(catalog.add_document(document.clone()).expect("valid document"))
        });
    cases.iter().fold(with_documents, |catalog, case| applied(catalog.save_case(case.clone()).expect("valid case")))
}

/// A tax office case with a letter, a due AI follow-up and a later manual one; a waiting objection; a closed claim.
fn screen_catalog() -> Catalog {
    let letter = document(4, "Request for receipts", "Tax office");
    let tax = case("tax", "Tax office asks for receipts", "Tax office")
        .with_entry(&letter.id, EntryDirection::Received, day(2026, 11, 4))
        .with_follow_up(follow_up("send", day(2026, 11, 10), FollowUpSource::Ai { document_id: letter.id.clone() }))
        .with_follow_up(follow_up("check", day(2026, 12, 1), FollowUpSource::Manual));
    let objection =
        case("objection", "Objection to the statement", "Landlord").with_status(CaseStatus::Waiting, day(2026, 10, 18));
    let claim = case("claim", "Damage claim", "Insurer").with_status(CaseStatus::Done, day(2026, 10, 2));
    catalog(&[letter], &[objection, tax, claim])
}

#[test]
fn the_active_list_starts_with_the_earliest_follow_up_and_says_what_comes_next() {
    let view = present_cases(&screen_catalog(), CasesFilterDto::Active, None, today());

    assert_eq!((view.active_count, view.done_count), (2, 1));
    let rows: Vec<(&str, &str, &str)> =
        view.cases.iter().map(|row| (row.id.as_str(), row.pill.label.as_str(), row.next_step.as_str())).collect();
    assert_eq!(
        rows,
        vec![
            ("tax", "Action needed", "Do send by 2026-11-10"),
            ("objection", "Waiting", "Waiting for an answer for 24 days"),
        ]
    );
    assert_eq!(view.detail.expect("the first case").id, "tax", "without a choice the first case shows");
}

#[test]
fn the_done_list_shows_the_chosen_case_or_falls_back_to_its_first() {
    let catalog = screen_catalog();
    let objection = CaseId::new("objection").expect("id");

    let done = present_cases(&catalog, CasesFilterDto::Done, Some(&objection), today());
    let chosen = present_cases(&catalog, CasesFilterDto::Active, Some(&objection), today());

    assert_eq!(done.cases[0].next_step, "Done on 2026-10-02");
    assert_eq!(done.detail.expect("detail").id, "claim");
    let detail = chosen.detail.expect("detail");
    assert_eq!(detail.summary, "Landlord · 0 documents · Waiting for an answer for 24 days");
    assert_eq!(detail.pill.status, CaseStatusDto::Waiting);
}

#[test]
fn the_detail_lists_the_ai_request_and_a_timeline_by_date() {
    let view = present_cases(&screen_catalog(), CasesFilterDto::Active, None, today());
    let detail = view.detail.expect("detail");

    assert_eq!(detail.requests.len(), 1);
    assert_eq!(detail.requests[0].document_name, "Request for receipts");
    assert_eq!(detail.requests[0].actions, vec!["Do send".to_owned()]);
    let timeline: Vec<(CaseTimelineKindDto, &str, bool)> =
        detail.timeline.iter().map(|item| (item.kind, item.marker.as_str(), item.due)).collect();
    assert_eq!(
        timeline,
        vec![
            (CaseTimelineKindDto::Received, "2026-11-04 · received", false),
            (CaseTimelineKindDto::FollowUp, "2026-11-10 · follow-up", true),
            (CaseTimelineKindDto::FollowUp, "2026-12-01 · follow-up", false),
        ]
    );
    assert_eq!(detail.timeline[1].detail.as_deref(), Some("Proposed by the AI from Request for receipts"));
    assert_eq!(detail.reminder.document_ids, vec!["d4".to_owned()]);
    assert_eq!(detail.reminder.recipient.as_deref(), Some("Tax office"));
}

#[test]
fn the_quick_look_offers_the_correspondents_cases_first_and_knows_about_links() {
    let letter = document(1, "Reminder", "Landlord");
    let reply = document(2, "Reply", "Landlord");
    let tax = case("tax", "Receipts", "Tax office");
    let objection = case("objection", "Objection", "Landlord");
    let member = case("member", "Statement", "Landlord").with_entry(&letter.id, EntryDirection::Received, today());
    let catalog = catalog(&[letter.clone(), reply.clone()], &[tax, objection, member]);
    let linked = applied(catalog.link(DocumentLink::new(letter.id.clone(), reply.id, None).expect("link")));

    let row = cases_of_document(&linked, &letter);

    assert_eq!(row.cases.iter().map(|case| case.id.as_str()).collect::<Vec<_>>(), vec!["member"]);
    assert_eq!(row.candidates.iter().map(|case| case.id.as_str()).collect::<Vec<_>>(), vec!["objection", "tax"]);
    assert!(row.has_links);
    assert!(!cases_of_document(&catalog, &letter).has_links);
}

#[test]
fn the_review_proposes_follow_ups_on_the_requested_date_or_in_two_weeks_and_the_waiting_case() {
    let letter = document(5, "Your objection", "Landlord");
    let invoice = document(6, "Invoice 114", "Plumber");
    let waiting = case("objection", "Objection", "landlord ").with_status(CaseStatus::Waiting, day(2026, 10, 1));
    let catalog = catalog(&[letter.clone(), invoice.clone()], &[waiting]);
    let mut suggestion = ClassificationSuggestion::empty(letter.id.clone(), "model");
    suggestion.requested_actions = vec![
        RequestedAction {
            text: "Send the invoices".to_owned(),
            due_date: Some(day(2026, 11, 30)),
            references: vec!["114".to_owned()],
        },
        RequestedAction { text: "Reply".to_owned(), due_date: None, references: Vec::new() },
    ];

    let review = review_cases(&catalog, &letter, Some(&suggestion), &[letter.id.clone(), invoice.id.clone()], today());

    let requested: Vec<(&str, &str)> =
        review.requested.iter().map(|action| (action.text.as_str(), action.date.as_str())).collect();
    assert_eq!(requested, vec![("Send the invoices", "2026-11-30"), ("Reply", "2026-11-25")]);
    assert_eq!(review.matches.iter().map(|matched| matched.document_id.as_str()).collect::<Vec<_>>(), vec!["d6"]);
    assert_eq!(review.open_case.expect("the landlord's case").id, "objection");
    assert!(review.pre_checked, "a waiting case is pre-checked");
    assert_eq!(review.new_title, "Your objection");
    let unrelated = review_cases(&catalog, &invoice, None, &[], today());
    assert!(unrelated.open_case.is_none() && unrelated.requested.is_empty() && !unrelated.pre_checked);
}

#[test]
fn add_document_offers_documents_outside_the_case_matching_the_search() {
    let inside = document(1, "Statement", "Landlord");
    let outside = document(2, "Reply from the landlord", "Landlord");
    let other = document(3, "Gym contract", "Gym");
    let member = case("member", "Statement", "Landlord").with_entry(&inside.id, EntryDirection::Received, today());
    let catalog = catalog(&[inside, outside, other], std::slice::from_ref(&member));

    let ids = |query: &str| -> Vec<String> {
        document_candidates(&catalog, &member, query).into_iter().map(|candidate| candidate.document_id).collect()
    };

    assert_eq!(ids("landlord"), vec!["d2".to_owned()]);
    assert_eq!(ids(""), vec!["d3".to_owned(), "d2".to_owned()]);
}

#[test]
fn dashboard_rows_of_cases_offer_their_real_actions() {
    let catalog = screen_catalog();
    let objection = CaseId::new("objection").expect("id");

    let follow_up: Vec<String> = follow_up_buttons(&objection, "key").into_iter().map(|button| button.label).collect();
    let waiting = waiting_buttons(&objection, "key", &catalog);

    assert_eq!(follow_up, vec!["Open case", "Done", "Snooze 7 days"]);
    assert_eq!(
        waiting.iter().map(|button| button.label.as_str()).collect::<Vec<_>>(),
        vec!["Open case", "Draft reminder", "Mark as done", "Dismiss"]
    );
    assert_eq!(
        waiting[1].action,
        DashboardAction::DraftReminder {
            case_id: "objection".to_owned(),
            document_ids: Vec::new(),
            recipient: Some("Landlord".to_owned()),
        }
    );
}
