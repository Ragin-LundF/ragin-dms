use std::sync::Arc;

use dms_application::classification::ListAiModels;
use dms_application::port::ai::{ClassificationFailure, ModelList};
use dms_application::port::state::AiSettings;

use crate::support::fake::ai::FakeAiServer;

fn setup() -> (Arc<FakeAiServer>, ListAiModels) {
    let server = Arc::new(FakeAiServer::default());
    let list_ai_models = ListAiModels::new(server.clone());
    (server, list_ai_models)
}

fn available(models: &[&str]) -> ModelList {
    ModelList::Available(models.iter().map(|model| (*model).to_owned()).collect())
}

#[test]
fn asks_the_server_with_the_given_settings_and_returns_its_models() {
    let (server, list_ai_models) = setup();
    let settings = AiSettings {
        base_url: "http://localhost:9999/v1".to_owned(),
        model: Some("sample-model".to_owned()),
        ..AiSettings::default()
    };
    server.answer.set(available(&["sample-model", "sample-vision"]));

    let models = list_ai_models.run(&settings);

    assert_eq!(models, available(&["sample-model", "sample-vision"]));
    assert_eq!(server.requests.get(), [settings]);
}

#[test]
fn an_unreachable_server_is_reported_as_unavailable_with_its_reason() {
    let (server, list_ai_models) = setup();
    let unavailable = ModelList::Unavailable {
        reason: ClassificationFailure::ProviderUnavailable,
        detail: Some("connection refused".to_owned()),
    };
    server.answer.set(unavailable.clone());

    assert_eq!(list_ai_models.run(&AiSettings::default()), unavailable);
}

#[test]
fn a_server_without_models_answers_with_an_empty_list() {
    let (server, list_ai_models) = setup();
    server.answer.set(available(&[]));

    assert_eq!(list_ai_models.run(&AiSettings::default()), available(&[]));
}
