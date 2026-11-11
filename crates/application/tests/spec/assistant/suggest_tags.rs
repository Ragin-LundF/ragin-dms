use dms_application::classification::SuggestOutcome;
use dms_application::port::assistant::AssistantAnswer;
use dms_application::port::state::TaskStore;

use super::add_tags;
use crate::support::TestLibrary;
use crate::support::fake::ai::FakeAssistant;
use crate::support::test_data::tag;

#[test]
fn suggested_tags_wait_for_review_without_the_ones_the_document_has() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/bill.pdf", "a");
    add_tags(&library, &document.id, &[tag("Tax")]);
    library
        .assistant
        .tags
        .set(AssistantAnswer::Tags { tags: vec![tag("Tax"), tag("Energy")], model: FakeAssistant::MODEL.into() });

    let outcome = library.suggest_tags().run(&document.id);

    let SuggestOutcome::Suggested(suggestion) = outcome else { panic!("expected Suggested, got {outcome:?}") };
    assert_eq!(suggestion.tags.iter().map(|it| it.name.clone()).collect::<Vec<_>>(), [tag("Energy")]);
    assert!(suggestion.tags[0].is_new);
    assert_eq!(library.tasks.suggestions().expect("readable"), [suggestion]);
}

#[test]
fn no_new_tags_is_answered_but_not_stored() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/bill.pdf", "a");
    library.assistant.tags.set(AssistantAnswer::Tags { tags: Vec::new(), model: FakeAssistant::MODEL.into() });

    let outcome = library.suggest_tags().run(&document.id);

    let SuggestOutcome::Suggested(suggestion) = outcome else { panic!("expected Suggested, got {outcome:?}") };
    assert_eq!(suggestion.tags, Vec::new());
    assert_eq!(library.tasks.suggestions().expect("readable"), Vec::new());
}

#[test]
fn a_file_the_ai_does_not_read_is_not_sent() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/notes.txt", "a");

    let outcome = library.suggest_tags().run(&document.id);

    assert!(matches!(outcome, SuggestOutcome::NotReadable { .. }), "got {outcome:?}");
    assert_eq!(library.assistant.requests.get().len(), 0);
}
