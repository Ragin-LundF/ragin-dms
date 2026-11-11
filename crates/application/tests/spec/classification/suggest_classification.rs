use dms_application::classification::SuggestOutcome;
use dms_application::port::ai::ClassificationResult;
use dms_application::port::state::TaskStore;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory, SuggestedTag};

use crate::support::TestLibrary;
use crate::support::fake::ai::FakeClassifier;
use crate::support::test_data::{category_id, tag};

#[test]
fn a_suggestion_waits_for_review_and_the_text_is_stored_in_every_folder() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/scan-001.pdf", "scan");
    library.classifier.result.set(Some(ClassificationResult::Suggested(ClassificationSuggestion {
        title: Some("Synthetic statement".to_owned()),
        category: Some(SuggestedCategory::Existing(category_id("unknown"))),
        tags: vec![SuggestedTag { name: tag("Tax"), is_new: false }],
        ..ClassificationSuggestion::empty(document.id.clone(), FakeClassifier::MODEL)
    })));

    let outcome = library.suggest_classification().run(&document.id);

    let SuggestOutcome::Suggested(suggestion) = outcome else { panic!("expected Suggested, got {outcome:?}") };
    assert_eq!(suggestion.category, None);
    assert_eq!(suggestion.tags.iter().map(|it| it.is_new).collect::<Vec<_>>(), [true]);
    assert_eq!(library.tasks.suggestions().expect("readable"), [suggestion]);
    assert_eq!(library.catalog().documents()[&document.id].classification, None);
    for folder in library.all_folders() {
        assert_eq!(library.folders.text_in(&folder, &document.id).as_deref(), Some("synthetic text"));
    }
}

#[test]
fn the_request_carries_the_library_s_file_names_as_examples() {
    let library = TestLibrary::new();
    library.open();
    library.add("letters/2024-01 first.pdf", "a");
    let document = library.add("inbox/scan-002.pdf", "b");

    let _ = library.suggest_classification().run(&document.id);

    let requests = library.classifier.requests.get();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].file_name_examples, ["2024-01 first.pdf"]);
    assert_eq!(requests[0].current_file_name, "scan-002.pdf");
}

#[test]
fn a_file_type_the_ai_cannot_read_is_never_sent() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("notes/plain.txt", "text");

    let outcome = library.suggest_classification().run(&document.id);

    assert_eq!(outcome, SuggestOutcome::NotReadable { detail: None });
    assert_eq!(library.classifier.requests.get().len(), 0);
}
