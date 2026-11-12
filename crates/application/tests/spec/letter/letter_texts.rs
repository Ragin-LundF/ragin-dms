use dms_application::letter::{detect_language, letter_document, template_draft};
use dms_application::port::assistant::{LetterKind, LetterLanguage, LetterParty, LetterRequest};
use dms_application::port::pdf::TextBlock;

use crate::support::test_data::date;

fn request(kind: LetterKind, language: LetterLanguage, facts: &[&str]) -> LetterRequest {
    LetterRequest {
        kind,
        sender: None,
        recipient: LetterParty::default(),
        subject_facts: facts.iter().map(|fact| (*fact).to_owned()).collect(),
        source_texts: Vec::new(),
        language,
    }
}

#[test]
fn german_text_makes_a_german_letter_and_anything_else_an_english_one() {
    let german = vec!["Bitte zahlen Sie den Betrag für die Rechnung bis zum Monatsende.".to_owned()];
    let english = vec!["Please pay the amount of your invoice by the end of the month.".to_owned()];
    let mixed = vec![german[0].clone(), english[0].clone(), english[0].clone()];

    assert_eq!(detect_language(&german), LetterLanguage::De);
    assert_eq!(detect_language(&english), LetterLanguage::En);
    assert_eq!(detect_language(&mixed), LetterLanguage::En);
    assert_eq!(detect_language(&[]), LetterLanguage::En);
}

#[test]
fn every_kind_has_a_template_in_both_languages() {
    let kinds =
        [LetterKind::Cancellation, LetterKind::Objection, LetterKind::Reminder, LetterKind::Request, LetterKind::Reply];
    for kind in kinds {
        let german = template_draft(&request(kind, LetterLanguage::De, &[]));
        let english = template_draft(&request(kind, LetterLanguage::En, &[]));

        assert!(german.body.starts_with("Sehr geehrte Damen und Herren,\n\n"), "{kind:?}: {}", german.body);
        assert!(german.body.ends_with("Mit freundlichen Grüßen"), "{kind:?}: {}", german.body);
        assert!(english.body.starts_with("Dear Sir or Madam,\n\n"), "{kind:?}: {}", english.body);
        assert_ne!(german.subject, english.subject);
    }
}

#[test]
fn the_facts_go_into_the_body_and_the_first_one_into_the_subject() {
    let draft = template_draft(&LetterRequest {
        sender: Some(LetterParty { name: "Erika Beispiel".to_owned(), address_lines: Vec::new() }),
        ..request(LetterKind::Cancellation, LetterLanguage::En, &["Contract: Internet 250", "Cancel by 2026-11-30"])
    });

    assert_eq!(draft.subject, "Cancellation: Contract: Internet 250");
    assert!(draft.body.contains("\n\nContract: Internet 250\nCancel by 2026-11-30\n\n"), "{}", draft.body);
    assert!(draft.body.ends_with("Kind regards\n\nErika Beispiel"), "{}", draft.body);
}

#[test]
fn a_letter_is_laid_out_with_sender_address_date_subject_and_one_paragraph_per_part() {
    let recipient =
        LetterParty { name: "Netz".to_owned(), address_lines: vec![" Postfach 1 ".to_owned(), String::new()] };

    let document = letter_document(
        None,
        &recipient,
        date(2026, 11, 3),
        " Reply ",
        "Dear Sir,\r\n\r\nline one\nline two\n\n\n",
        LetterLanguage::En,
    );

    assert_eq!(
        document.blocks,
        vec![
            TextBlock::Paragraph("Netz\nPostfach 1".to_owned()),
            TextBlock::Spacer,
            TextBlock::Paragraph("3 November 2026".to_owned()),
            TextBlock::Spacer,
            TextBlock::Heading("Reply".to_owned()),
            TextBlock::Paragraph("Dear Sir,".to_owned()),
            TextBlock::Paragraph("line one\nline two".to_owned()),
        ]
    );
}
