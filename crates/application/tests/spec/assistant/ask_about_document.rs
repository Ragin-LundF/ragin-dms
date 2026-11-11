use dms_application::assistant::AnswerOutcome;
use dms_application::port::ai::DocumentContent;
use dms_application::port::assistant::{ChatRole, ChatTurn};

use crate::support::TestLibrary;

fn turn(role: ChatRole, text: &str) -> ChatTurn {
    ChatTurn { role, text: text.to_owned() }
}

#[test]
fn the_conversation_goes_with_the_question_and_the_document_is_read_once() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/bill.pdf", "a");
    let first = vec![turn(ChatRole::User, "Who sent it?")];
    let mut second = first.clone();
    second.extend([turn(ChatRole::Assistant, "Sample Energy"), turn(ChatRole::User, "When?")]);
    let ask = library.ask_about_document();

    let _ = ask.run(&document.id, &first);
    library.reader.content.set(DocumentContent::Text("changed, but the document is unchanged".to_owned()));
    let outcome = ask.run(&document.id, &second);

    let AnswerOutcome::Answered { text, .. } = outcome else { panic!("expected Answered, got {outcome:?}") };
    assert_eq!(text, "A synthetic answer");
    assert_eq!(library.assistant.conversations.get(), [first, second]);
    let texts: Vec<String> = library
        .assistant
        .requests
        .get()
        .into_iter()
        .map(|request| match request.content {
            DocumentContent::Text(text) => text,
            other => panic!("expected text, got {other:?}"),
        })
        .collect();
    assert_eq!(texts, ["synthetic text", "synthetic text"]);
}
