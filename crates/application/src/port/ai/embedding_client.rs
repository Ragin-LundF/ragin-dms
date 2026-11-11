use crate::port::ai::EmbeddingResult;

/// Turns texts into embedding vectors on the AI server (`/v1/embeddings` of LM Studio or Ollama).
pub trait EmbeddingClient: Send + Sync {
    /// The embedding model that [`Self::embed`] uses now, or `None` if none is set and the server offers none.
    fn model(&self) -> Option<String>;

    fn embed(&self, texts: &[String]) -> EmbeddingResult;
}
