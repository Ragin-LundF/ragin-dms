//! Validation of the records added in 2.1 and the keys of their tasks.

use dms_domain::cases::{CaseId, FollowUp, FollowUpId, FollowUpSource};
use dms_domain::contract::{ContractId, ContractProposal, ContractTerms, Notice, NoticeUnit, ProposalReason};
use dms_domain::emergency::{EmergencyEntry, EmergencyFolder, EmergencyGroup, EmergencyItem, EmergencyItemId};
use dms_domain::expectation::{ExpectedDocument, Rhythm};
use dms_domain::library::LibraryFolderId;
use dms_domain::settings::OwnerDetails;
use dms_domain::task::OpenTask;
use dms_domain::tax::{TaxArea, TaxChecklistItem, TaxItemId, TaxItemState, TaxYear};

use crate::test_data::{self, category_id, date, document_id};

#[test]
fn a_contract_needs_a_name_and_sensible_terms() {
    let contract = test_data::contract("c-1", "Electricity");
    let zero_notice =
        ContractTerms { notice: Some(Notice { count: 0, unit: NoticeUnit::Months }), ..ContractTerms::default() };
    let zero_renewal = ContractTerms { renews_by_months: Some(0), ..ContractTerms::default() };

    assert!(dms_domain::contract::Contract { terms: zero_notice, ..contract.clone() }.validate().is_err());
    assert!(dms_domain::contract::Contract { terms: zero_renewal, ..contract.clone() }.validate().is_err());
    assert!(dms_domain::contract::Contract { note: Some(" ".to_owned()), ..contract.clone() }.validate().is_err());
    assert!(dms_domain::contract::Contract::new(ContractId::new("c-2").expect("id"), "\t", contract.kind).is_err());
    assert!(ContractId::new(" ").is_err());
}

#[test]
fn a_case_needs_a_title_distinct_follow_ups_and_a_reminder_of_at_least_a_day() {
    let case = test_data::case("case-1", "Objection");
    let follow_up = FollowUp {
        id: FollowUpId::new("f-1").expect("id"),
        date: date(2026, 11, 30),
        text: "Send the receipts".to_owned(),
        done: false,
        source: FollowUpSource::Manual,
    };

    assert_eq!(case.remind_after_days, 21);
    assert!(
        dms_domain::cases::Case { follow_ups: vec![follow_up.clone(), follow_up.clone()], ..case.clone() }
            .validate()
            .is_err()
    );
    assert!(dms_domain::cases::Case { remind_after_days: 0, ..case.clone() }.validate().is_err());
    assert!(
        dms_domain::cases::Case { follow_ups: vec![FollowUp { text: String::new(), ..follow_up }], ..case.clone() }
            .validate()
            .is_err()
    );
}

#[test]
fn a_tax_year_needs_distinct_items_with_labels() {
    let item = TaxChecklistItem {
        id: TaxItemId::new("i-1").expect("id"),
        area: TaxArea::Income,
        label: "Lohnsteuerbescheinigung".to_owned(),
        correspondent: None,
        document_id: None,
        state: TaxItemState::Missing,
        note: None,
        added_by_user: true,
    };

    assert!(TaxYear { items: vec![item.clone()], ..TaxYear::new(2025) }.validate().is_ok());
    assert!(TaxYear { items: vec![item.clone(), item.clone()], ..TaxYear::new(2025) }.validate().is_err());
    assert!(
        TaxYear { items: vec![TaxChecklistItem { label: " ".to_owned(), ..item }], ..TaxYear::new(2025) }
            .validate()
            .is_err()
    );
}

#[test]
fn emergency_items_and_owner_details_need_their_texts() {
    let contact = |name: &str| EmergencyItem {
        id: EmergencyItemId::new("e-1").expect("id"),
        group: EmergencyGroup::Contacts,
        entry: EmergencyEntry::Contact {
            name: name.to_owned(),
            role: None,
            phone: Some("0123".to_owned()),
            email: None,
            note: None,
        },
        included: true,
        recorded_hash: None,
    };
    let owner = OwnerDetails { name: "Alex Example".to_owned(), address_lines: vec![], email: None, phone: None };

    assert!(EmergencyFolder { items: vec![contact("Sam")], ..EmergencyFolder::default() }.validate().is_ok());
    assert!(EmergencyFolder { items: vec![contact(" ")], ..EmergencyFolder::default() }.validate().is_err());
    assert!(EmergencyFolder { refresh_months: 0, ..EmergencyFolder::default() }.validate().is_err());
    assert!(owner.validate().is_ok());
    assert!(OwnerDetails { name: " ".to_owned(), ..owner.clone() }.validate().is_err());
    assert!(OwnerDetails { address_lines: vec![String::new()], ..owner }.validate().is_err());
}

#[test]
fn the_new_tasks_have_stable_keys_that_change_with_their_date() {
    let doc = document_id("doc-1");
    let case_id = CaseId::new("case-1").expect("id");
    let expected = ExpectedDocument {
        correspondent: "Employer Ltd".to_owned(),
        category_id: Some(category_id("salary")),
        expected_on: date(2026, 10, 4),
        rhythm: Rhythm::Monthly,
        first_seen_year: 2021,
    };
    let keys: Vec<String> = [
        OpenTask::BillOverdue { document_id: doc.clone(), due: date(2026, 11, 8) },
        OpenTask::ExpectedDocumentMissing(expected.clone()),
        OpenTask::ExpectedDocumentMissing(ExpectedDocument { category_id: None, ..expected }),
        OpenTask::ReviewContractProposal(ContractProposal {
            correspondent: "FitWerk".to_owned(),
            document_ids: vec![doc.clone()],
            reason: ProposalReason::DeadlineField,
        }),
        OpenTask::ContractCancelBy { contract_id: ContractId::new("c-1").expect("id"), date: date(2026, 11, 30) },
        OpenTask::FollowUpDue {
            case_id: case_id.clone(),
            follow_up_id: FollowUpId::new("f-1").expect("id"),
            date: date(2026, 11, 30),
        },
        OpenTask::CaseWaitingTooLong { case_id, since: date(2026, 10, 1) },
        OpenTask::DamagedCopy {
            document_id: doc.clone(),
            folder_id: LibraryFolderId::new("ssd").expect("id"),
            source_folder_id: LibraryFolderId::new("primary").expect("id"),
        },
        OpenTask::UnreadableDocument { document_id: doc },
        OpenTask::EmergencyFolderRefresh { due: date(2026, 11, 14) },
    ]
    .iter()
    .map(|task| task.key().value().to_owned())
    .collect();

    assert_eq!(
        keys,
        vec![
            "bill:doc-1:2026-11-08",
            "expected:employer ltd:salary:2026-10-04",
            "expected:employer ltd:-:2026-10-04",
            "contract-proposal:fitwerk",
            "contract-cancel:c-1:2026-11-30",
            "follow-up:case-1:f-1:2026-11-30",
            "case-waiting:case-1:2026-10-01",
            "damaged:doc-1:ssd",
            "unreadable:doc-1",
            "emergency-refresh:2026-11-14",
        ]
    );
}
