use std::collections::HashMap;

use dms_application::ask::{AskOutcome, RelatedOutcome};
use dms_application::catalog::CatalogEdit;
use dms_application::port::storage::FolderOperation;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::query::DocumentQuery;

use crate::ask::kit::AskKit;
use crate::support::TestLibrary;
use crate::support::test_data::done;

fn library_with_texts() -> (AskKit, Document, Document) {
    let library = TestLibrary::new();
    library.open();
    let power = library.add("power.pdf", "a");
    let car = library.add("car.pdf", "b");
    library.reader.texts.set(HashMap::from([
        ("power.pdf".to_owned(), "Sample Energy electricity bill, meter reading and power used in March".to_owned()),
        ("car.pdf".to_owned(), "Car insurance policy for the family car".to_owned()),
    ]));
    (AskKit::new(library), power, car)
}

fn indexed(progress: &[dms_application::ask::IndexProgress]) -> Vec<bool> {
    progress.iter().map(|step| step.indexed).collect()
}

#[test]
fn indexing_writes_one_entry_per_document_everywhere_and_skips_what_is_current() {
    let (kit, power, _) = library_with_texts();

    let first = kit.index();
    let second = kit.index();

    assert_eq!(indexed(&first), vec![true, true]);
    assert_eq!(indexed(&second), vec![false, false]);
    for folder in kit.library.all_folders() {
        assert!(
            dms_application::port::index::VectorIndex::read(kit.library.index.as_ref(), &folder, &power.id).is_some()
        );
    }
}

#[test]
fn a_changed_title_another_model_or_an_offline_folder_are_handled() {
    let (kit, power, _) = library_with_texts();
    let library = &kit.library;
    kit.index();
    let metadata = DocumentMetadata { title: Some("Power".to_owned()), ..DocumentMetadata::default() };
    done(library.edit_catalog().run(&CatalogEdit::UpdateMetadata { document_id: power.id.clone(), metadata }));
    library.folders.make_unreachable(&library.sync_b);

    let after_title = kit.index();
    library.embeddings.model.set("other-embed".to_owned());
    let after_model = kit.index();

    // By path: car.pdf stays as it was, power.pdf has a new title.
    assert_eq!(indexed(&after_title), vec![false, true]);
    assert_eq!(indexed(&after_model), vec![true, true]);
    let pending = library.journal.operations.get();
    assert!(pending.iter().any(|pending| pending.operation == FolderOperation::CopyIndex(power.id.clone())));
}

#[test]
fn without_the_embedding_server_indexing_stops_with_the_reason() {
    let (kit, _, _) = library_with_texts();
    kit.library.embeddings.available.set(false);

    let progress = kit.index();

    assert_eq!(progress.len(), 1);
    assert_eq!(progress[0].failure.as_deref(), Some("offline"));
}

#[test]
fn the_closest_passages_go_to_the_chat_model_and_citations_map_back_to_documents() {
    let (kit, power, _) = library_with_texts();
    kit.index();
    kit.library.library_assistant.answer.set("The meter showed it [1]; nothing else [7].".to_owned());

    let outcome = kit.ask("How much electricity did the meter show?", &DocumentQuery::default());

    let AskOutcome::Answered { citations, used_vectors, .. } = outcome else {
        panic!("expected Answered, got {outcome:?}")
    };
    assert_eq!(citations.len(), 1);
    assert_eq!(citations[0].document_id, power.id);
    assert_eq!(citations[0].number, 1);
    assert!(used_vectors);
    let sources = kit.library.library_assistant.sources.get();
    assert_eq!(sources.len(), 1);
    assert!(sources[0][0].text.contains("electricity bill"));
    assert_eq!(sources[0][0].title, power.path.file_name());
}

#[test]
fn without_an_index_the_words_of_the_question_still_find_passages() {
    let (kit, _, car) = library_with_texts();
    kit.library.embeddings.available.set(false);

    let outcome = kit.ask("Which insurance policy do we have?", &DocumentQuery::default());

    let AskOutcome::Answered { citations, used_vectors, .. } = outcome else {
        panic!("expected Answered, got {outcome:?}")
    };
    assert_eq!(citations.len(), 1);
    assert_eq!(citations[0].document_id, car.id);
    assert!(!used_vectors);
}

#[test]
fn nothing_fitting_is_an_honest_not_found_without_asking_the_ai() {
    let (kit, _, _) = library_with_texts();
    kit.index();

    let scope = DocumentQuery { text: Some("no document has this".to_owned()), ..DocumentQuery::default() };
    let outcome = kit.ask("doctor", &scope);

    assert_eq!(outcome, AskOutcome::NotFound);
    assert_eq!(kit.library.library_assistant.sources.get().len(), 0);
}

#[test]
fn related_documents_are_the_ones_closest_in_meaning() {
    let (kit, power, car) = library_with_texts();
    let meter = kit.library.add("meter.pdf", "c");
    kit.library.reader.set_text("meter.pdf", "Electricity meter reading, power");
    kit.index();

    let related = kit.related_documents().run(&power.id, 5);

    let RelatedOutcome::Found(related) = related else { panic!("expected Found, got {related:?}") };
    assert_eq!(related, vec![meter.id]);
    assert!(!related.contains(&car.id));
}

#[test]
fn the_metadata_hash_is_java_string_hash_code_in_hex() {
    use dms_application::ask::IndexLibrary;

    assert_eq!(IndexLibrary::hash_of("hello"), "5e918d2");
    assert_eq!(IndexLibrary::hash_of("polygenelubricants"), "80000000");
    assert_eq!(IndexLibrary::hash_of(""), "0");
}
