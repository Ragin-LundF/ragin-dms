use dms_application::assistant::AnswerOutcome;

use crate::support::TestLibrary;

#[test]
fn a_summary_is_answered_for_a_readable_document() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/bill.pdf", "a");

    let outcome = library.summarize_document().run(&document.id);

    let AnswerOutcome::Answered { text, .. } = outcome else { panic!("expected Answered, got {outcome:?}") };
    assert_eq!(text, "A synthetic summary");
    let requests = library.assistant.requests.get();
    assert_eq!(requests.iter().map(|it| it.file_name.as_str()).collect::<Vec<_>>(), ["bill.pdf"]);
}
