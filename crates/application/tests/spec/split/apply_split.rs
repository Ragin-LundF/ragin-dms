use dms_application::common::OperationOutcome;
use dms_application::port::state::TaskStore;
use dms_application::split::{ApplySplitInput, ApplySplitOutcome};
use dms_domain::classification::SuggestedCategory;
use dms_domain::document::{Document, DocumentId};
use dms_domain::split::{SplitPart, SplitProblem};

use super::kit::SplitKit;
use crate::support::test_data::date;

fn input(document: &Document, parts: Vec<SplitPart>) -> ApplySplitInput {
    ApplySplitInput {
        document_id: document.id.clone(),
        parts,
        model: "split-model".to_owned(),
        trash_original: false,
        confirmed: true,
    }
}

fn invoice(first: usize, last: usize) -> SplitPart {
    SplitPart {
        title: Some("Water bill".to_owned()),
        category: Some(SuggestedCategory::New { name: "Bills".to_owned(), parent_id: None }),
        correspondent: Some("Stadtwerke Beispielstadt".to_owned()),
        document_date: Some(date(2026, 3, 2)),
        confidence: Some(0.9),
        ..SplitPart::pages(first, last)
    }
}

fn split(outcome: ApplySplitOutcome) -> (Vec<DocumentId>, Vec<DocumentId>) {
    match outcome {
        ApplySplitOutcome::Split { outcome: OperationOutcome::Done { .. }, parts, without_suggestion } => {
            (parts, without_suggestion)
        }
        other => panic!("expected a split, got {other:?}"),
    }
}

#[test]
fn every_part_with_ai_values_waits_for_review_with_them_as_a_suggestion_and_nothing_is_applied() {
    let kit = SplitKit::new();
    let stack = kit.add("Inbox/stack.pdf", "a|b|c");

    let (parts, without_suggestion) =
        split(kit.apply_split().run(&input(&stack, vec![invoice(0, 1), SplitPart::pages(2, 2)])));

    assert_eq!(parts.len(), 2);
    assert_eq!(without_suggestion, vec![parts[1].clone()]);
    let suggestions = kit.tasks.suggestions().expect("suggestions readable");
    assert_eq!(suggestions.len(), 1);
    let suggestion = &suggestions[0];
    assert_eq!(suggestion.document_id, parts[0]);
    assert_eq!(suggestion.model, "split-model");
    assert_eq!(suggestion.title.as_deref(), Some("Water bill"));
    assert_eq!(suggestion.correspondent.as_deref(), Some("Stadtwerke Beispielstadt"));
    assert_eq!(suggestion.document_date, Some(date(2026, 3, 2)));
    assert_eq!(suggestion.confidence, Some(0.9));
    let catalog = kit.catalog();
    let first = catalog.documents().get(&parts[0]).expect("first part catalogued");
    assert_eq!(first.metadata.title, None, "a suggestion is not applied");
    assert_eq!(kit.folders.content(&kit.sync_a, first.path.value()).as_deref(), Some("a|b"));
    assert!(catalog.documents().contains_key(&stack.id), "the original stays unless the user chose otherwise");
    assert_eq!(catalog.categories().len(), 0, "a suggested category is not created");
}

#[test]
fn the_original_goes_to_the_trash_only_when_the_user_chose_it() {
    let kit = SplitKit::new();
    let stack = kit.add("stack.pdf", "a|b");

    let outcome = kit.apply_split().run(&ApplySplitInput {
        trash_original: true,
        ..input(&stack, vec![SplitPart::pages(0, 0), SplitPart::pages(1, 1)])
    });

    let (parts, _) = split(outcome);
    let catalog = kit.catalog();
    assert!(!catalog.documents().contains_key(&stack.id));
    assert!(parts.iter().all(|part| catalog.documents().contains_key(part)));
}

#[test]
fn without_confirmation_or_with_parts_that_do_not_cover_the_pages_nothing_is_split() {
    let kit = SplitKit::new();
    let stack = kit.add("stack.pdf", "a|b|c");
    let parts = vec![SplitPart::pages(0, 0), SplitPart::pages(1, 2)];

    let unconfirmed = kit.apply_split().run(&ApplySplitInput { confirmed: false, ..input(&stack, parts) });
    let gap = kit.apply_split().run(&input(&stack, vec![SplitPart::pages(0, 0), SplitPart::pages(2, 2)]));
    let whole = kit.apply_split().run(&input(&stack, vec![SplitPart::pages(0, 2)]));

    assert_eq!(unconfirmed, ApplySplitOutcome::NotSplit(OperationOutcome::NotConfirmed));
    assert_eq!(gap, ApplySplitOutcome::InvalidParts(SplitProblem::Gap));
    assert_eq!(whole, ApplySplitOutcome::NothingToSplit);
    assert_eq!(kit.catalog().documents().len(), 1);
}
