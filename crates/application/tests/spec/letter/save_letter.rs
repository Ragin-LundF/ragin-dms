use dms_application::common::OperationOutcome;
use dms_application::letter::{SaveLetterInput, SaveLetterOutcome};
use dms_application::port::assistant::{LetterLanguage, LetterParty};
use dms_domain::document::DocumentId;
use dms_domain::taxonomy::Category;

use super::kit::LetterKit;
use crate::support::test_data::{category_id, date, document_id, done, path};

fn input(document_ids: Vec<DocumentId>, folder: &str, subject: &str) -> SaveLetterInput {
    SaveLetterInput {
        document_ids,
        folder: path(folder),
        sender: Some(LetterParty { name: "Erika Beispiel".to_owned(), address_lines: vec!["Weg 1".to_owned()] }),
        recipient: LetterParty {
            name: " Netz Neu ".to_owned(),
            address_lines: vec!["Kundenservice".to_owned(), "12345 Hafenstadt".to_owned()],
        },
        subject: subject.to_owned(),
        body: "Sehr geehrte Damen und Herren,\n\nhiermit kündige ich.\n\nMit freundlichen Grüßen\nErika Beispiel"
            .to_owned(),
        language: LetterLanguage::De,
    }
}

fn saved(outcome: SaveLetterOutcome) -> (DocumentId, String) {
    match outcome {
        SaveLetterOutcome::Saved { document_id, path, .. } => (document_id, path.value().to_owned()),
        SaveLetterOutcome::Refused(outcome) => panic!("expected Saved, got {outcome:?}"),
    }
}

#[test]
fn a_saved_letter_is_a_pdf_in_every_folder_catalogued_and_linked_to_its_sources() {
    let kit = LetterKit::new();
    let source = kit.add_from("contracts/price.pdf", "Netz Neu", date(2025, 1, 5), "text");
    done(kit.store.commit(|catalog| {
        catalog.add_category(Category::new(category_id("cat-letter"), "letter", None, None).expect("category"))
    }));

    let (id, saved_path) =
        saved(kit.save_letter().run(&input(vec![source.id.clone()], "contracts", "Kündigung Vertrag 4711")));

    assert_eq!(saved_path, "contracts/2024-03-01 Kündigung Vertrag 4711.pdf");
    for folder in kit.all_folders() {
        let content = kit.folders.content(&folder, &saved_path).expect("the letter is in every folder");
        assert!(content.contains("Erika Beispiel · Weg 1"), "{content}");
        assert!(content.contains("Netz Neu\nKundenservice\n12345 Hafenstadt"), "{content}");
        assert!(content.contains("01.03.2024"), "{content}");
        assert!(content.contains("# Kündigung Vertrag 4711"), "{content}");
        assert!(content.contains("hiermit kündige ich."), "{content}");
    }
    let catalog = kit.catalog();
    let letter = catalog.documents().get(&id).expect("catalogued");
    assert_eq!(letter.metadata.title.as_deref(), Some("Kündigung Vertrag 4711"));
    assert_eq!(letter.metadata.correspondent.as_deref(), Some("Netz Neu"));
    assert_eq!(letter.metadata.document_date, Some(date(2024, 3, 1)));
    assert_eq!(letter.category_id, Some(category_id("cat-letter")));
    let links = catalog.links_of(&id);
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].other_than(&id), &source.id);
    assert_eq!(links[0].label(), Some("letter"));
}

#[test]
fn a_taken_name_gets_a_counter_and_forbidden_characters_are_replaced() {
    let kit = LetterKit::new();
    kit.folders.put(&kit.primary, "2024-03-01 Objection 1-2.pdf", "existing");

    let (id, saved_path) = saved(kit.save_letter().run(&input(Vec::new(), "", "Objection 1/2")));

    assert_eq!(saved_path, "2024-03-01 Objection 1-2 (2).pdf");
    assert_eq!(kit.folders.content(&kit.primary, "2024-03-01 Objection 1-2.pdf").as_deref(), Some("existing"));
    let letter = kit.catalog().documents().get(&id).cloned().expect("catalogued");
    assert_eq!(letter.category_id, None, "without a Letter category it stays uncategorized");
    assert_eq!(letter.metadata.title.as_deref(), Some("Objection 1/2"));
}

#[test]
fn a_blank_subject_or_an_unknown_source_saves_nothing() {
    let kit = LetterKit::new();
    let before = kit.folders.paths(&kit.primary);

    let blank = kit.save_letter().run(&input(Vec::new(), "", "  "));
    let unknown = kit.save_letter().run(&input(vec![document_id("missing")], "", "Reply"));

    assert!(matches!(blank, SaveLetterOutcome::Refused(OperationOutcome::InvalidInput(_))), "{blank:?}");
    assert_eq!(unknown, SaveLetterOutcome::Refused(OperationOutcome::NotFound));
    assert_eq!(kit.folders.paths(&kit.primary), before);
    assert_eq!(kit.catalog().documents().len(), 0);
}

#[test]
fn a_letter_saved_while_a_sync_folder_is_offline_waits_for_it_in_the_journal() {
    let kit = LetterKit::new();
    kit.folders.make_unreachable(&kit.sync_b);

    let outcome = kit.save_letter().run(&input(Vec::new(), "letters", "Request"));

    let SaveLetterOutcome::Saved { pending_folders, path, .. } = outcome else {
        panic!("expected Saved, got {outcome:?}")
    };
    assert_eq!(pending_folders.len(), 1);
    assert!(kit.folders.content(&kit.sync_a, path.value()).is_some());
    assert!(kit.folders.content(&kit.sync_b, path.value()).is_none());
}
