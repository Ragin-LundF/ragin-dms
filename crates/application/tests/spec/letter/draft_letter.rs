use dms_application::letter::{DraftLetterInput, DraftLetterOutcome, DraftSource};
use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{LetterAnswer, LetterKind, LetterLanguage, LetterParty};
use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;

use super::kit::LetterKit;
use crate::support::fake::assistant::FakeLetterWriter;
use crate::support::test_data::{date, document_id, path};

const GERMAN_TEXT: &str = "Sehr geehrte Kundin, die Preise für Ihren Vertrag ändern sich zum 1. Januar.";
const ENGLISH_TEXT: &str = "Dear customer, the prices of your contract change on 1 January.";

fn input(kind: LetterKind, document_ids: Vec<DocumentId>) -> DraftLetterInput {
    DraftLetterInput {
        kind,
        document_ids,
        recipient: None,
        subject_facts: vec!["Customer no. 4711".to_owned()],
        sender: Some(LetterParty { name: "Erika Beispiel".to_owned(), address_lines: vec!["Weg 1".to_owned()] }),
    }
}

#[test]
fn without_ai_the_template_answers_in_the_language_of_the_sources_from_the_newest_correspondent() {
    let kit = LetterKit::new();
    let older = kit.add_from("old/price-2024.pdf", "Netz Alt", date(2024, 1, 5), GERMAN_TEXT);
    let newer = kit.add_from("contracts/price-2025.pdf", "Netz Neu", date(2025, 1, 5), GERMAN_TEXT);

    let outcome = kit.draft_letter().run(&input(LetterKind::Cancellation, vec![older.id, newer.id]));

    let DraftLetterOutcome::Drafted { draft, recipient, language, folder, source } = outcome else {
        panic!("expected a draft, got {outcome:?}");
    };
    assert_eq!(source, DraftSource::Template { ai_failure: None });
    assert_eq!(language, LetterLanguage::De);
    assert_eq!(recipient, LetterParty { name: "Netz Neu".to_owned(), address_lines: Vec::new() });
    assert_eq!(folder, path("contracts"));
    assert_eq!(draft.subject, "Kündigung: Customer no. 4711");
    assert!(draft.body.starts_with("Sehr geehrte Damen und Herren,"), "{}", draft.body);
    assert!(draft.body.ends_with("Mit freundlichen Grüßen\n\nErika Beispiel"), "{}", draft.body);
    assert_eq!(kit.writer.requests.get(), Vec::new(), "the AI is never asked while it is off");
}

#[test]
fn with_ai_the_writer_drafts_from_the_source_texts_and_the_given_recipient() {
    let kit = LetterKit::new();
    kit.enable_ai();
    let source = kit.add_from("letter.pdf", "Netz Neu", date(2025, 1, 5), ENGLISH_TEXT);

    let outcome = kit.draft_letter().run(&DraftLetterInput {
        recipient: Some("Customer Care".to_owned()),
        ..input(LetterKind::Objection, vec![source.id])
    });

    let DraftLetterOutcome::Drafted { draft, recipient, source, language, .. } = outcome else {
        panic!("expected a draft, got {outcome:?}");
    };
    assert_eq!(source, DraftSource::Ai { model: FakeLetterWriter::MODEL.to_owned() });
    assert_eq!(draft.subject, "AI subject");
    assert_eq!(recipient.name, "Customer Care");
    assert_eq!(language, LetterLanguage::En);
    let request = kit.writer.requests.get().pop().expect("asked once");
    assert_eq!(request.kind, LetterKind::Objection);
    assert_eq!(request.subject_facts, vec!["Customer no. 4711".to_owned()]);
    assert_eq!(request.source_texts, vec![format!("letter.pdf (2025-01-05):\n{ENGLISH_TEXT}")]);
}

#[test]
fn the_owner_details_never_reach_the_ai_and_the_draft_is_signed_with_the_owner_name_here() {
    let kit = LetterKit::new();
    kit.enable_ai();
    let source = kit.add_from("letter.pdf", "Netz Neu", date(2025, 1, 5), ENGLISH_TEXT);

    let outcome = kit.draft_letter().run(&input(LetterKind::Objection, vec![source.id]));

    let DraftLetterOutcome::Drafted { draft, .. } = outcome else {
        panic!("expected a draft, got {outcome:?}");
    };
    let request = kit.writer.requests.get().pop().expect("asked once");
    assert_eq!(request.sender, None);
    assert!(draft.body.ends_with("\nErika Beispiel"), "signed body: {}", draft.body);
    assert!(!format!("{request:?}").contains("Weg 1"));
}

#[test]
fn the_source_texts_together_stay_within_the_ai_input_budget() {
    let kit = LetterKit::new();
    kit.enable_ai();
    kit.settings.preferences.update(|preferences| preferences.ai.input_characters = 1_000);
    let long = "a".repeat(800);
    let first = kit.add_from("one.pdf", "Netz", date(2025, 2, 1), &long);
    let second = kit.add_from("two.pdf", "Netz", date(2025, 1, 1), &long);
    let third = kit.add_from("three.pdf", "Netz", date(2024, 1, 1), &long);

    let _ = kit.draft_letter().run(&input(LetterKind::Reply, vec![third.id, second.id, first.id]));

    let texts = kit.writer.requests.get().pop().expect("asked once").source_texts;
    assert_eq!(texts.len(), 2, "the oldest gets no budget left");
    assert!(texts[0].starts_with("one.pdf"), "newest first: {}", texts[0]);
    let sent: usize = texts.iter().map(|text| text.matches('a').count()).sum();
    assert_eq!(sent, 1_000);
}

#[test]
fn a_failed_ai_falls_back_to_the_template_and_says_why() {
    let kit = LetterKit::new();
    kit.enable_ai();
    kit.writer.answer.set(LetterAnswer::Failed { reason: ClassificationFailure::ProviderUnavailable, detail: None });

    let outcome = kit.draft_letter().run(&input(LetterKind::Reminder, Vec::new()));

    let DraftLetterOutcome::Drafted { draft, source, folder, language, recipient } = outcome else {
        panic!("expected a draft, got {outcome:?}");
    };
    assert_eq!(source, DraftSource::Template { ai_failure: Some(ClassificationFailure::ProviderUnavailable) });
    assert_eq!(draft.subject, "Reminder: Customer no. 4711");
    assert_eq!(language, LetterLanguage::En, "no source text: English");
    assert_eq!(folder, RelativePath::root());
    assert_eq!(recipient.name, "");
}

#[test]
fn an_unknown_source_document_is_not_found() {
    let kit = LetterKit::new();

    let outcome = kit.draft_letter().run(&input(LetterKind::Request, vec![document_id("missing")]));

    assert_eq!(outcome, DraftLetterOutcome::NotFound);
}
