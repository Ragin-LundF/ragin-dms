#![allow(clippy::expect_used)]

use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{
    LetterAnswer, LetterDraft, LetterKind, LetterLanguage, LetterParty, LetterRequest, LetterWriter,
};
use dms_application::port::state::AiSettings;

use crate::test_support::{StubServer, adapters_with, chat_answer, local_settings, ok, unreachable_base_url};

fn request() -> LetterRequest {
    LetterRequest {
        kind: LetterKind::Cancellation,
        sender: Some(LetterParty { name: "Erika Beispiel".to_owned(), address_lines: vec!["Weg 1".to_owned()] }),
        recipient: LetterParty { name: "Netz GmbH".to_owned(), address_lines: Vec::new() },
        subject_facts: vec!["Customer no. 4711".to_owned()],
        source_texts: vec!["Preisänderung zum 1. Januar".to_owned()],
        language: LetterLanguage::De,
    }
}

fn draft_with(content: &str) -> (LetterAnswer, StubServer) {
    let answer = chat_answer(content);
    let server = StubServer::start(move |_| ok(answer.clone()));
    let result = server.adapters(local_settings(), None).letter_writer.draft(&request());
    (result, server)
}

fn reason_of(answer: LetterAnswer) -> ClassificationFailure {
    match answer {
        LetterAnswer::Failed { reason, .. } => reason,
        LetterAnswer::Drafted { draft, .. } => panic!("drafted: {draft:?}"),
    }
}

#[test]
fn a_valid_answer_becomes_a_trimmed_draft_and_the_request_carries_language_facts_and_documents() {
    let (answer, server) = draft_with(
        r#"{"subject":" Kündigung Kundennr. 4711 ","body":"Sehr geehrte Damen und Herren,\n\nhiermit kündige ich.\n"}"#,
    );

    assert_eq!(
        answer,
        LetterAnswer::Drafted {
            draft: LetterDraft {
                subject: "Kündigung Kundennr. 4711".to_owned(),
                body: "Sehr geehrte Damen und Herren,\n\nhiermit kündige ich.".to_owned(),
            },
            model: "sample-model".to_owned(),
        }
    );
    let body = &server.chat_bodies()[0];
    assert_eq!(body["response_format"]["json_schema"]["name"], "letter");
    let system = body["messages"][0]["content"].as_str().expect("system text");
    let user = body["messages"][1]["content"].as_str().expect("request text");
    assert!(system.contains("Write the letter in German"), "{system}");
    assert!(user.contains("Kind of letter: cancellation of a contract"), "{user}");
    assert!(user.contains("Sender: Erika Beispiel, Weg 1") && user.contains("Recipient: Netz GmbH"), "{user}");
    assert!(user.contains("Customer no. 4711") && user.contains("Preisänderung zum 1. Januar"), "{user}");
}

#[test]
fn a_fenced_answer_with_a_sentence_around_it_still_counts() {
    let (answer, _server) =
        draft_with("Here it is:\n```json\n{\"subject\":\"Reply\",\"body\":\"Dear Sir or Madam,\"}\n```");

    assert!(matches!(answer, LetterAnswer::Drafted { .. }), "{answer:?}");
}

#[test]
fn malformed_off_schema_or_blank_answers_are_an_invalid_response() {
    for content in [
        "I cannot write this letter.",
        r#"{"subject": "Reply", "body": "#,
        r#"{"title":"Reply","text":"Dear Sir or Madam,"}"#,
        r#"{"subject":"Reply","body":42}"#,
        r#"{"subject":"  ","body":"Dear Sir or Madam,"}"#,
    ] {
        let (answer, _server) = draft_with(content);

        assert_eq!(reason_of(answer), ClassificationFailure::InvalidResponse, "{content}");
    }
}

#[test]
fn an_unreachable_server_is_a_typed_failure() {
    let settings = AiSettings { base_url: unreachable_base_url(), ..local_settings() };

    let answer = adapters_with(settings, None, &unreachable_base_url()).letter_writer.draft(&request());

    assert_eq!(reason_of(answer), ClassificationFailure::ProviderUnavailable);
}
