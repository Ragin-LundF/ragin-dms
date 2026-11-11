use dms_domain::document::DocumentId;

/// Progress of building the index: `done` of `total` documents; the last one was `indexed` anew or was current
/// already. `failure` names why the AI server could not make embeddings, which stops the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexProgress {
    pub done: usize,
    pub total: usize,
    pub document_id: DocumentId,
    pub indexed: bool,
    pub failure: Option<String>,
}
