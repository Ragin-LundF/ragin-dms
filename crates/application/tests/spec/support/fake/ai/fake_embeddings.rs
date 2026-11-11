use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::port::ai::{ClassificationFailure, EmbeddingClient, EmbeddingResult};
use dms_domain::text;

use crate::support::Shared;

/// Embeddings by counting the words of [`Self::VOCABULARY`] in a text, so texts about the same things get close
/// vectors, deterministically. Fails while `available` is off.
pub struct FakeEmbeddings {
    pub available: Shared<bool>,
    pub model: Shared<String>,
    calls: AtomicUsize,
}

impl FakeEmbeddings {
    pub const VOCABULARY: [&'static str; 8] =
        ["electricity", "power", "meter", "insurance", "car", "policy", "tax", "doctor"];

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn vector_of(text: &str) -> Vec<f32> {
        let lower = text::lowercase(text);
        #[allow(clippy::cast_precision_loss)] // word counts of test texts
        Self::VOCABULARY.iter().map(|word| lower.matches(word).count() as f32).collect()
    }
}

impl Default for FakeEmbeddings {
    fn default() -> Self {
        Self { available: Shared::new(true), model: Shared::new("fake-embed".to_owned()), calls: AtomicUsize::new(0) }
    }
}

impl EmbeddingClient for FakeEmbeddings {
    fn model(&self) -> Option<String> {
        self.available.get().then(|| self.model.get())
    }

    fn embed(&self, texts: &[String]) -> EmbeddingResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if !self.available.get() {
            return EmbeddingResult::Failed {
                reason: ClassificationFailure::ProviderUnavailable,
                detail: Some("offline".to_owned()),
            };
        }
        EmbeddingResult::Vectors {
            model: self.model.get(),
            vectors: texts.iter().map(|text| Self::vector_of(text)).collect(),
        }
    }
}
