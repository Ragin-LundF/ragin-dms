use dms_application::ask::{AskOutcome, Citation};
use dms_application::port::ai::ClassificationFailure;
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::document::DocumentId;
use dms_domain::query::{DocumentQuery, SavedSearch, SavedSearchId};

use super::*;

fn catalog_with_search(name: &str, query: DocumentQuery) -> Catalog {
    let catalog = Catalog::empty(LibraryId::new("lib").expect("id"));
    let mut settings = catalog.settings().clone();
    settings.saved_searches.push(SavedSearch::new(SavedSearchId::new("s1").expect("id"), name, query).expect("search"));
    match catalog.change_settings(settings).expect("valid") {
        CatalogUpdate::Applied(next) => next,
        CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
    }
}

#[test]
fn scopes_are_the_whole_library_then_the_smart_folders() {
    let query = DocumentQuery { text: Some("insurance".to_owned()), ..DocumentQuery::default() };
    let catalog = catalog_with_search("Contracts", query.clone());

    assert_eq!(scope_labels(Some(&catalog)), ["Whole library", "Smart folder: Contracts"]);
    assert_eq!(scope_labels(None), ["Whole library"]);
    assert_eq!(scope_query(Some(&catalog), 1), query);
    assert_eq!(scope_query(Some(&catalog), 0), DocumentQuery::default());
    assert_eq!(scope_query(Some(&catalog), 5), DocumentQuery::default());
}

#[test]
fn an_answer_lists_its_citations_and_says_when_only_words_were_searched() {
    let outcome = AskOutcome::Answered {
        text: "It ends in May [1].".to_owned(),
        citations: vec![Citation {
            number: 1,
            document_id: DocumentId::new("d1").expect("id"),
            title: "Policy".to_owned(),
        }],
        model: "m".to_owned(),
        used_vectors: false,
    };

    let entry = entry_of("When does it end?", &outcome);

    assert_eq!(entry.answer.as_deref(), Some("It ends in May [1]."));
    assert_eq!(entry.citations[0].document_id, "d1");
    assert_eq!(entry.note.as_deref(), Some("Searched by words only; update the index to search by meaning"));
}

#[test]
fn questions_without_an_answer_get_a_note() {
    let failed = AskOutcome::Failed { reason: ClassificationFailure::Timeout, detail: None };

    assert_eq!(
        entry_of("q", &AskOutcome::NotFound).note.as_deref(),
        Some("Nothing in your documents fits this question.")
    );
    assert_eq!(
        entry_of("q", &failed).note.as_deref(),
        Some("The AI took too long; try again, or raise the timeout in Settings › AI")
    );
}

#[test]
fn index_message_names_the_count_or_the_failure() {
    assert_eq!(index_message(3, None), UiMessage::info("The index is up to date; 3 documents were read anew"));
    assert_eq!(
        index_message(0, Some("The AI server returned no vectors")),
        UiMessage::error("The index could not be built: The AI server returned no vectors")
    );
}
