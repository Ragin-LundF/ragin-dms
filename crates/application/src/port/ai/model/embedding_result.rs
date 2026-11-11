use crate::port::ai::ClassificationFailure;

/// The embeddings of some texts, one vector per text in their order, from `model`; or why there are none.
#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingResult {
    Vectors { model: String, vectors: Vec<Vec<f32>> },
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
