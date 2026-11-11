#![allow(clippy::expect_used)]

use dms_application::port::ai::{ClassificationFailure, EmbeddingClient, EmbeddingResult};
use dms_application::port::state::{AiProvider, AiSettings};

use crate::test_support::{Reply, StubServer, adapters_with, ok, unreachable_base_url};

fn chosen(model: &str) -> AiSettings {
    AiSettings { embedding_model: Some(model.to_owned()), ..AiSettings::default() }
}

fn texts(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn reason_of(result: &EmbeddingResult) -> ClassificationFailure {
    match *result {
        EmbeddingResult::Failed { reason, .. } => reason,
        EmbeddingResult::Vectors { .. } => panic!("vectors"),
    }
}

#[test]
fn texts_are_embedded_with_the_chosen_model_and_returned_in_their_order() {
    let server =
        StubServer::start(|_| ok(r#"{"data":[{"index":1,"embedding":[0.5,0.5]},{"index":0,"embedding":[1.0,0.0]}]}"#));

    let result = server.adapters(chosen("sample-embed"), None).embeddings.embed(&texts(&["first", "second"]));

    assert_eq!(
        result,
        EmbeddingResult::Vectors { model: "sample-embed".to_owned(), vectors: vec![vec![1.0, 0.0], vec![0.5, 0.5]] }
    );
    let sent = server.received()[0].json();
    assert_eq!(sent["model"], "sample-embed");
    assert_eq!(sent["input"].as_array().expect("input").len(), 2);
}

#[test]
fn without_a_chosen_model_the_first_embedding_model_of_the_server_is_used() {
    let server = StubServer::start(|_| ok(r#"{"data":[{"id":"chat-model"},{"id":"text-embedding-sample"}]}"#));

    assert_eq!(
        server.adapters(AiSettings::default(), None).embeddings.model().as_deref(),
        Some("text-embedding-sample")
    );
}

#[test]
fn a_blank_chosen_model_falls_back_to_the_servers_embedding_model_whatever_its_case() {
    let server = StubServer::start(|_| ok(r#"{"data":[{"id":"chat-model"},{"id":"Sample-EMBED-Text"}]}"#));

    assert_eq!(server.adapters(chosen("  "), None).embeddings.model().as_deref(), Some("Sample-EMBED-Text"));
}

#[test]
fn a_refusing_server_or_a_wrong_number_of_vectors_is_a_failure() {
    let refusing = StubServer::start(|_| Reply::Status(404, "{}".to_owned()));
    let short = StubServer::start(|_| ok(r#"{"data":[]}"#));

    assert_eq!(
        reason_of(&refusing.adapters(chosen("e"), None).embeddings.embed(&texts(&["a"]))),
        ClassificationFailure::ProviderUnavailable
    );
    assert_eq!(
        reason_of(&short.adapters(chosen("e"), None).embeddings.embed(&texts(&["a"]))),
        ClassificationFailure::InvalidResponse
    );
}

#[test]
fn without_any_embedding_model_the_texts_are_not_sent_and_the_provider_is_unavailable() {
    let server = StubServer::start(|_| ok(r#"{"data":[{"id":"chat-model"}]}"#));

    let result = server.adapters(AiSettings::default(), None).embeddings.embed(&texts(&["a"]));

    assert_eq!(reason_of(&result), ClassificationFailure::ProviderUnavailable);
    assert_eq!(server.paths(), ["/v1/models"]);
}

#[test]
fn a_server_that_cannot_be_reached_is_unavailable_with_or_without_a_chosen_model() {
    for settings in [chosen("e"), AiSettings::default()] {
        let settings = AiSettings { base_url: unreachable_base_url(), ..settings };

        let result = adapters_with(settings, None, &unreachable_base_url()).embeddings.embed(&texts(&["a"]));

        assert_eq!(reason_of(&result), ClassificationFailure::ProviderUnavailable);
    }
}

#[test]
fn a_refusing_servers_status_and_a_shortened_answer_are_the_detail() {
    let server = StubServer::start(|_| Reply::Status(503, "x".repeat(1_000)));

    let result = server.adapters(chosen("e"), None).embeddings.embed(&texts(&["a"]));

    assert_eq!(
        result,
        EmbeddingResult::Failed {
            reason: ClassificationFailure::ProviderUnavailable,
            detail: Some(format!("HTTP 503: {}", "x".repeat(300))),
        }
    );
}

#[test]
fn a_missing_or_empty_vector_makes_the_whole_answer_invalid_not_a_partial_result() {
    let answers = [
        r#"{"data":[{"index":0,"embedding":[1.0]},{"index":1}]}"#,
        r#"{"data":[{"index":0,"embedding":[1.0]},{"index":1,"embedding":[]}]}"#,
        r#"{"data":[{"index":0,"embedding":[1.0]}]}"#,
        r#"{"model":"e"}"#,
    ];
    for answer in answers {
        let server = StubServer::start(move |_| ok(answer));

        let result = server.adapters(chosen("e"), None).embeddings.embed(&texts(&["first", "second"]));

        assert_eq!(reason_of(&result), ClassificationFailure::InvalidResponse, "{answer}");
    }
}

#[test]
fn a_base_url_ending_with_a_slash_still_reaches_the_embeddings_endpoint() {
    let server = StubServer::start(|_| ok(r#"{"data":[{"index":0,"embedding":[1.0]}]}"#));
    let settings = AiSettings { base_url: format!(" {}/ ", server.base_url()), ..chosen("e") };

    let result = adapters_with(settings, None, &server.anthropic_base_url()).embeddings.embed(&texts(&["a"]));

    assert!(matches!(result, EmbeddingResult::Vectors { .. }), "{result:?}");
    assert_eq!(server.paths(), ["/v1/embeddings"]);
}

#[test]
fn an_answer_that_is_no_json_or_not_the_embeddings_schema_is_invalid() {
    let answers = [
        "<html>Internal error</html>",
        "",
        r#"{"data":{"index":0,"embedding":[1.0]}}"#,
        r#"{"data":[{"index":0,"embedding":["one"]}]}"#,
        r#"{"data":[{"index":"first","embedding":[1.0]}]}"#,
    ];
    for answer in answers {
        let server = StubServer::start(move |_| ok(answer));

        let result = server.adapters(chosen("e"), None).embeddings.embed(&texts(&["a"]));

        assert_eq!(reason_of(&result), ClassificationFailure::InvalidResponse, "{answer}");
    }
}

#[test]
fn vectors_that_cannot_be_matched_to_the_texts_make_the_answer_invalid() {
    let answers = [
        r#"{"data":[{"index":0,"embedding":[1.0]},{"index":0,"embedding":[2.0]}]}"#,
        r#"{"data":[{"embedding":[1.0]},{"embedding":[2.0]}]}"#,
        r#"{"data":[{"index":1,"embedding":[1.0]},{"index":2,"embedding":[2.0]}]}"#,
        r#"{"data":[{"index":0,"embedding":[1.0,2.0]},{"index":1,"embedding":[2.0]}]}"#,
    ];
    for answer in answers {
        let server = StubServer::start(move |_| ok(answer));

        let result = server.adapters(chosen("e"), None).embeddings.embed(&texts(&["first", "second"]));

        assert_eq!(reason_of(&result), ClassificationFailure::InvalidResponse, "{answer}");
    }
}

#[test]
fn while_anthropic_answers_the_chats_embeddings_go_to_the_local_server_without_the_key() {
    let server = StubServer::start(|_| ok(r#"{"data":[{"index":0,"embedding":[1.0]}]}"#));
    let settings = AiSettings { provider: AiProvider::Anthropic, ..chosen("sample-embed") };

    let result = server.adapters(settings, Some("sample-key")).embeddings.embed(&texts(&["first"]));

    assert!(matches!(result, EmbeddingResult::Vectors { .. }), "{result:?}");
    let received = server.received();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].path, "/v1/embeddings");
    assert_eq!(received[0].header("x-api-key"), None);
}
