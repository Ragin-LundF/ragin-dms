use std::sync::Arc;

use dms_domain::document::DocumentId;

use crate::ask::{RelatedOutcome, VectorSearch};
use crate::library::CatalogStore;

const MIN_SIMILARITY: f64 = 0.5;

/// The documents closest in meaning to one document, by the average of their vectors in the index.
pub struct RelatedDocuments {
    store: Arc<CatalogStore>,
    vectors: Arc<VectorSearch>,
}

impl RelatedDocuments {
    /// Kotlin's default `limit`.
    pub const DEFAULT_LIMIT: usize = 5;

    pub fn new(store: Arc<CatalogStore>, vectors: Arc<VectorSearch>) -> Self {
        Self { store, vectors }
    }

    pub fn run(&self, document_id: &DocumentId, limit: usize) -> RelatedOutcome {
        let Some(library) = self.store.current() else {
            return RelatedOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return RelatedOutcome::NotFound;
        };
        let folder = library.folders.primary();
        let Some((model, centroid)) = self.vectors.centroid(folder, document) else {
            return RelatedOutcome::Found(Vec::new());
        };
        let mut related: Vec<(DocumentId, f64)> = library
            .catalog
            .documents()
            .values()
            .filter(|other| other.id != *document_id)
            .filter_map(|other| {
                let (other_model, other_centroid) = self.vectors.centroid(folder, other)?;
                let comparable = other_model == model && other_centroid.len() == centroid.len();
                comparable.then(|| (other.id.clone(), VectorSearch::cosine(&centroid, &other_centroid)))
            })
            .filter(|(_, similarity)| *similarity >= MIN_SIMILARITY)
            .collect();
        related.sort_by(|first, second| second.1.total_cmp(&first.1));
        RelatedOutcome::Found(related.into_iter().take(limit).map(|(id, _)| id).collect())
    }
}
