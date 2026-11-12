use dms_application::common::OperationOutcome;
use dms_application::contract::{AttachDocumentsInput, ContractDraft};
use dms_domain::catalog::CatalogRejection;
use dms_domain::contract::{ContractId, ContractTerms, Notice, NoticeUnit, PricePoint};
use dms_domain::money::BillingPeriod;
use dms_domain::task::OpenTask;
use indexmap::IndexSet;

use super::kit::{ContractKit, draft, euros};
use crate::support::test_data::{date, done};

#[test]
fn a_new_contract_from_a_proposal_gets_an_id_its_documents_price_and_terms() {
    let kit = ContractKit::new();
    let letter = kit.add("contracts/internet.pdf", "internet letter");
    let terms = ContractTerms {
        term_end: Some(date(2024, 12, 31)),
        renews_by_months: Some(12),
        notice: Some(Notice { count: 1, unit: NoticeUnit::Months }),
        cancellable_any_time: false,
    };

    let id = kit.record(&ContractDraft {
        price: Some(euros("44.99", BillingPeriod::Monthly)),
        terms: terms.clone(),
        document_ids: vec![letter.id.clone()],
        ..draft("Internet")
    });

    let contract = kit.catalog().contracts().get(&id).cloned().expect("recorded");
    assert_eq!(contract.name, "Internet");
    assert_eq!(contract.price, Some(euros("44.99", BillingPeriod::Monthly)));
    assert_eq!(contract.price_history, Vec::<PricePoint>::new());
    assert_eq!(contract.document_ids, IndexSet::from([letter.id]));
    assert_eq!(contract.terms.next_cancel_by(date(2024, 3, 1)), Some(date(2024, 11, 30)));
    assert_eq!(kit.activity_log.entries.get().first().map(|entry| entry.subject.clone()), Some("Internet".to_owned()));
}

#[test]
fn editing_keeps_documents_history_and_end_and_a_dated_price_change_goes_into_the_history() {
    let kit = ContractKit::new();
    let letter = kit.add("internet.pdf", "letter");
    let id = kit.record(&ContractDraft {
        price: Some(euros("39.99", BillingPeriod::Monthly)),
        price_valid_from: Some(date(2023, 1, 1)),
        document_ids: vec![letter.id.clone()],
        ..draft("Internet")
    });
    done(kit.end_contract().run(&id, Some(date(2025, 1, 31))));

    kit.record(&ContractDraft {
        id: Some(id.clone()),
        price: Some(euros("44.99", BillingPeriod::Monthly)),
        price_valid_from: Some(date(2024, 1, 1)),
        note: Some("Price rose in January".to_owned()),
        ..draft("Internet 250")
    });

    let contract = kit.catalog().contracts().get(&id).cloned().expect("still there");
    assert_eq!(contract.name, "Internet 250");
    assert_eq!(contract.price, Some(euros("44.99", BillingPeriod::Monthly)));
    assert_eq!(
        contract.price_history.iter().map(|point| point.valid_from).collect::<Vec<_>>(),
        vec![date(2023, 1, 1), date(2024, 1, 1)]
    );
    assert_eq!(contract.document_ids, IndexSet::from([letter.id]));
    assert_eq!(contract.ended_on, Some(date(2025, 1, 31)));
}

#[test]
fn saving_an_unknown_contract_or_a_blank_name_changes_nothing() {
    let kit = ContractKit::new();
    let before = kit.catalog();

    let unknown =
        kit.save_contract().run(&ContractDraft { id: Some(ContractId::new("gone").expect("id")), ..draft("Internet") });
    let blank = kit.save_contract().run(&draft(" "));

    assert_eq!(unknown.outcome, OperationOutcome::Rejected(CatalogRejection::ContractNotFound));
    assert!(matches!(blank.outcome, OperationOutcome::InvalidInput(_)), "{:?}", blank.outcome);
    assert_eq!(kit.catalog(), before);
}

#[test]
fn deleting_needs_the_confirmation_and_keeps_the_documents() {
    let kit = ContractKit::new();
    let letter = kit.add("internet.pdf", "letter");
    let id = kit.record(&ContractDraft { document_ids: vec![letter.id.clone()], ..draft("Internet") });

    let refused = kit.delete_contract().run(&id, false);
    assert_eq!(refused, OperationOutcome::NotConfirmed);
    assert!(kit.catalog().contracts().contains_key(&id));

    done(kit.delete_contract().run(&id, true));
    assert!(!kit.catalog().contracts().contains_key(&id));
    assert!(kit.catalog().documents().contains_key(&letter.id));
}

#[test]
fn ending_and_resuming_a_contract() {
    let kit = ContractKit::new();
    let id = kit.record(&draft("Gym"));

    done(kit.end_contract().run(&id, Some(date(2024, 3, 1))));
    assert_eq!(kit.catalog().contracts()[&id].ended_on, Some(date(2024, 3, 1)));
    assert!(!kit.catalog().contracts()[&id].is_running(date(2024, 3, 1)));

    done(kit.end_contract().run(&id, None));
    assert_eq!(kit.catalog().contracts()[&id].ended_on, None);
    assert_eq!(
        kit.end_contract().run(&ContractId::new("gone").expect("id"), None),
        OperationOutcome::Rejected(CatalogRejection::ContractNotFound)
    );
}

#[test]
fn documents_are_attached_and_detached() {
    let kit = ContractKit::new();
    let first = kit.add("first.pdf", "first");
    let second = kit.add("second.pdf", "second");
    let id = kit.record(&draft("Internet"));
    let input = |document_ids, detach| AttachDocumentsInput { contract_id: id.clone(), document_ids, detach };

    done(kit.attach_documents().run(&input(vec![first.id.clone(), second.id.clone()], false)));
    done(kit.attach_documents().run(&input(vec![first.id.clone()], true)));

    assert_eq!(kit.catalog().contracts()[&id].document_ids, IndexSet::from([second.id]));
}

#[test]
fn a_dismissed_proposal_is_kept_once_per_correspondent_and_disappears_from_the_tasks() {
    let kit = ContractKit::new();

    done(kit.dismiss_proposal().run(" Gym Club "));
    done(kit.dismiss_proposal().run("GYM CLUB"));

    assert_eq!(kit.catalog().settings().dismissed_contract_proposals, vec!["Gym Club".to_owned()]);
    assert_eq!(kit.dismiss_proposal().run("  "), OperationOutcome::NotFound);
    let proposals = dms_domain::contract::ContractPlanner::due_tasks(&kit.catalog(), date(2024, 3, 1));
    assert!(!proposals.iter().any(|task| matches!(task, OpenTask::ReviewContractProposal(_))));
}

#[test]
fn an_accepted_price_point_keeps_everything_else_and_becomes_the_price_once_valid() {
    let kit = ContractKit::new();
    let letter = kit.add("price.pdf", "letter");
    let id = kit.record(&ContractDraft { price: Some(euros("39.99", BillingPeriod::Monthly)), ..draft("Internet") });
    let contract = kit.catalog().contracts()[&id].clone();
    let point = PricePoint {
        valid_from: date(2024, 1, 1),
        price: euros("44.99", BillingPeriod::Monthly),
        document_id: Some(letter.id.clone()),
    };

    kit.record(&ContractDraft { added_prices: vec![point.clone()], ..ContractDraft::of(&contract) });

    let saved = kit.catalog().contracts()[&id].clone();
    assert_eq!(saved.price_history, vec![point]);
    assert_eq!(saved.price, Some(euros("44.99", BillingPeriod::Monthly)));
    assert_eq!(saved.name, contract.name);
}
