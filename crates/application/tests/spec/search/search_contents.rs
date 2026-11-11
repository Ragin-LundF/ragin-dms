use std::collections::HashMap;

use dms_application::port::storage::TextStore;
use dms_application::search::{FindOutcome, SearchOutcome};
use dms_domain::document::{Document, DocumentId};
use dms_domain::query::DocumentQuery;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::path;

fn found(library: &TestLibrary, query: &str) -> IndexSet<DocumentId> {
    match library.search_contents().run(query) {
        SearchOutcome::Found(ids) => ids,
        SearchOutcome::LibraryNotOpen => panic!("expected Found"),
    }
}

fn matched(library: &TestLibrary, query: &DocumentQuery) -> Vec<Document> {
    match library.find_documents().run(query) {
        FindOutcome::Found(documents) => documents,
        FindOutcome::LibraryNotOpen => panic!("expected Found"),
    }
}

fn texts(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries.iter().map(|(name, text)| ((*name).to_owned(), (*text).to_owned())).collect()
}

#[test]
fn nothing_is_searched_while_no_library_is_open() {
    let library = TestLibrary::new();

    assert_eq!(library.search_contents().run("a"), SearchOutcome::LibraryNotOpen);
    assert_eq!(library.find_documents().run(&DocumentQuery::default()), FindOutcome::LibraryNotOpen);
}

#[test]
fn documents_are_found_by_the_text_of_their_file_ignoring_case() {
    let library = TestLibrary::new();
    library.open();
    let bill = library.add("bills/power.pdf", "a");
    library.add("bills/water.pdf", "b");
    library.reader.texts.set(texts(&[("power.pdf", "Sample Energy Ltd, meter reading 4711")]));

    assert_eq!(found(&library, "  METER reading "), IndexSet::from([bill.id]));
    assert_eq!(found(&library, "insurance"), IndexSet::new());
}

#[test]
fn the_text_the_ai_transcribed_is_found_too() {
    let library = TestLibrary::new();
    library.open();
    let scan = library.add("scans/receipt.png", "a");
    library.folders.write(&library.primary, &scan.id, "Sample Bakery receipt").expect("written");

    assert_eq!(found(&library, "bakery"), IndexSet::from([scan.id]));
}

#[test]
fn each_file_is_read_once_until_it_changes() {
    let library = TestLibrary::new();
    library.open();
    library.add("a.pdf", "a");
    library.add("b.pdf", "b");

    library.search_contents().run("first");
    library.search_contents().run("second");

    assert_eq!(library.reader.text_reads(), 2);
}

#[test]
fn an_empty_query_finds_nothing() {
    let library = TestLibrary::new();
    library.open();
    library.add("a.pdf", "a");

    assert_eq!(found(&library, " "), IndexSet::new());
}

#[test]
fn a_note_counts_as_text_of_its_document() {
    let library = TestLibrary::new();
    library.open();
    let bill = library.add("bills/power.pdf", "a");
    library.store.commit(|catalog| Ok(catalog.set_note(&bill.id, Some("Disputed with the supplier"))));

    assert_eq!(found(&library, "disputed"), IndexSet::from([bill.id]));
}

#[test]
fn a_query_filters_by_its_conditions_and_then_by_the_content_term() {
    let library = TestLibrary::new();
    library.open();
    let power = library.add("bills/power.pdf", "a");
    library.add("bills/water.pdf", "b");
    library.add("letters/power-note.pdf", "c");
    library.reader.texts.set(texts(&[("power.pdf", "meter reading"), ("power-note.pdf", "meter reading")]));

    let in_bills = DocumentQuery {
        folder: Some(path("bills")),
        content_text: Some("METER".to_owned()),
        ..DocumentQuery::default()
    };

    assert_eq!(matched(&library, &in_bills), vec![power]);
    assert_eq!(matched(&library, &DocumentQuery::default()).len(), 3);
}
