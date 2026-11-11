use std::sync::Arc;

use dms_application::port::ai::{ClassificationFailure, EmbeddingClient, EmbeddingResult, ModelList};
use dms_application::port::state::{AiProvider, AiSettings};
use dms_domain::text;
use serde_json::json;

use crate::client::dto::EmbeddingsResponseDto;
use crate::client::{ChatTransport, failure_of, local_base_of, request_timeout, shortened};

/// Embeddings on any server with the OpenAI `/embeddings` API (LM Studio, Ollama's `/v1`). The model is the one
/// chosen in the AI settings, or else the first embedding model the server lists. Anthropic has no embeddings, so
/// this always uses the local server, also while Anthropic answers the chats; the key is never sent here.
pub struct OpenAiEmbeddings {
    transport: Arc<ChatTransport>,
}

impl OpenAiEmbeddings {
    pub(crate) fn new(transport: Arc<ChatTransport>) -> Self {
        Self { transport }
    }

    fn local(&self) -> AiSettings {
        AiSettings { provider: AiProvider::Local, ..self.transport.settings() }
    }

    fn model_of(&self, settings: &AiSettings) -> Option<String> {
        if let Some(model) = settings.embedding_model.as_ref().filter(|model| !text::is_blank(model)) {
            return Some(model.clone());
        }
        let ModelList::Available(models) = self.transport.models(settings) else {
            return None;
        };
        models.into_iter().find(|model| text::lowercase(model).contains("embed"))
    }
}

impl EmbeddingClient for OpenAiEmbeddings {
    fn model(&self) -> Option<String> {
        self.model_of(&self.local())
    }

    fn embed(&self, texts: &[String]) -> EmbeddingResult {
        let settings = self.local();
        let Some(model) = self.model_of(&settings) else {
            return failed(
                ClassificationFailure::ProviderUnavailable,
                "No embedding model: load one in LM Studio or choose it in the AI settings",
            );
        };
        let url = format!("{}/embeddings", local_base_of(&settings));
        let body = json!({"model": model, "input": texts});
        let answer = match self.transport.send(&url, Some(&body), request_timeout(&settings, 0), None) {
            Ok(answer) => answer,
            Err(error) => return failed(failure_of(&error), error.to_string()),
        };
        if !answer.is_success() {
            return failed(
                ClassificationFailure::ProviderUnavailable,
                format!("HTTP {}: {}", answer.status, shortened(&answer.body)),
            );
        }
        let mut data = match answer.json::<EmbeddingsResponseDto>() {
            Ok(response) => response.data,
            Err(detail) => return failed(ClassificationFailure::InvalidResponse, detail),
        };
        data.sort_by_key(|vector| vector.index);
        // One vector per text, by index, all of one size; anything else cannot be matched to the texts.
        let in_order = data.len() == texts.len()
            && data.iter().enumerate().all(|(position, vector)| usize::try_from(vector.index) == Ok(position));
        let size = data.first().map_or(0, |vector| vector.embedding.len());
        let complete =
            in_order && data.iter().all(|vector| !vector.embedding.is_empty() && vector.embedding.len() == size);
        if !complete {
            return failed(
                ClassificationFailure::InvalidResponse,
                format!("The server returned {} vectors", data.len()),
            );
        }
        EmbeddingResult::Vectors { model, vectors: data.into_iter().map(|vector| vector.embedding).collect() }
    }
}

fn failed(reason: ClassificationFailure, detail: impl Into<String>) -> EmbeddingResult {
    EmbeddingResult::Failed { reason, detail: Some(detail.into()) }
}
