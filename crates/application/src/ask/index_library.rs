use std::sync::Arc;

use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::text;

use crate::ask::{DocumentTexts, IndexProgress, TextChunks};
use crate::common::Progress;
use crate::library::CatalogStore;
use crate::port::ai::{ClassificationFailure, EmbeddingClient, EmbeddingResult};
use crate::port::index::{DocumentVectors, IndexedChunk, VectorIndex};
use crate::port::storage::FolderOperation;
use crate::sync::Replicator;

const BATCH: usize = 16;

/// Builds the embedding index of the library, document by document: only what is new or changed (other file, other
/// metadata, other embedding model) is embedded again. Each entry goes to `.dms-index/` of the primary and is
/// replicated. Cancelling stops it between two documents; running it again goes on where it stopped.
pub struct IndexLibrary {
    store: Arc<CatalogStore>,
    texts: Arc<DocumentTexts>,
    embeddings: Arc<dyn EmbeddingClient>,
    index: Arc<dyn VectorIndex>,
    replicator: Arc<Replicator>,
}

impl IndexLibrary {
    pub fn new(
        store: Arc<CatalogStore>,
        texts: Arc<DocumentTexts>,
        embeddings: Arc<dyn EmbeddingClient>,
        index: Arc<dyn VectorIndex>,
        replicator: Arc<Replicator>,
    ) -> Self {
        Self { store, texts, embeddings, index, replicator }
    }

    /// Reports one [`IndexProgress`] per document and returns the last one.
    pub fn run(&self, progress: &dyn Progress<IndexProgress>) -> Option<IndexProgress> {
        let library = self.store.current()?;
        let model = self.embeddings.model();
        let mut documents: Vec<&Document> = library.catalog.documents().values().collect();
        documents.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
        let mut last = None;
        for (position, document) in documents.iter().enumerate() {
            if progress.is_cancelled() {
                break;
            }
            let header = TextChunks::header(document, &library.catalog);
            let existing = self.index.read(library.folders.primary(), &document.id);
            let metadata_hash = Self::hash_of(&header);
            let current = match (&model, &existing) {
                (Some(model), Some(existing)) => existing.is_current(model, &document.content_hash, &metadata_hash),
                _ => false,
            };
            let failure = if current { None } else { self.build(&library.catalog, document, &header) };
            let step = IndexProgress {
                done: position + 1,
                total: documents.len(),
                document_id: document.id.clone(),
                indexed: !current && failure.is_none(),
                failure,
            };
            progress.report(&step);
            let failed = step.failure.is_some();
            last = Some(step);
            if failed {
                break;
            }
        }
        last
    }

    /// A short fingerprint of the header; a change of title, sender, category or tags rebuilds the entry. Java's
    /// `Integer.toHexString(text.hashCode())`, kept for `.dms-index/` parity.
    pub fn hash_of(text: &str) -> String {
        let hash = text.encode_utf16().fold(0_i32, |hash, unit| hash.wrapping_mul(31).wrapping_add(i32::from(unit)));
        format!("{:x}", hash.cast_unsigned())
    }

    /// Embeds and stores one document; returns why it failed, or `None`.
    fn build(&self, catalog: &Catalog, document: &Document, header: &str) -> Option<String> {
        let library = self.store.current()?;
        let text = self.texts.of(library.folders.primary(), document);
        let chunks = TextChunks::of(&text);
        let mut vectors: Vec<Vec<f32>> = Vec::new();
        let mut model = None;
        for batch in chunks.chunks(BATCH) {
            let inputs: Vec<String> = batch.iter().map(|chunk| format!("{header}\n\n{}", chunk.text)).collect();
            match self.embeddings.embed(&inputs) {
                EmbeddingResult::Failed { reason, detail } => {
                    return Some(detail.unwrap_or_else(|| kotlin_name(reason)));
                }
                EmbeddingResult::Vectors { model: used, vectors: batch_vectors } => {
                    vectors.extend(batch_vectors);
                    model = Some(used);
                }
            }
        }
        let Some(dimensions) = vectors.first().map(Vec::len) else {
            return Some("The AI server returned no vectors".to_owned());
        };
        let entry = DocumentVectors {
            document_id: document.id.clone(),
            model: model.expect("invariant: a vector came with its model"),
            dimensions,
            content_hash: document.content_hash.clone(),
            metadata_hash: Self::hash_of(&TextChunks::header(document, catalog)),
            chunks: chunks
                .iter()
                .zip(vectors)
                .map(|(chunk, vector)| IndexedChunk { start: chunk.start, end: chunk.end, vector })
                .collect(),
        };
        // Kotlin ignored the replication result too: a failed primary write is simply indexed again next time.
        let _ = self.replicator.replicate(
            &library.folders,
            |primary| self.index.write(primary, &entry),
            &FolderOperation::CopyIndex(document.id.clone()),
        );
        None
    }
}

/// The Kotlin enum constant name, which the progress shows when the provider gave no detail.
fn kotlin_name(reason: ClassificationFailure) -> String {
    match reason {
        ClassificationFailure::ProviderUnavailable => "PROVIDER_UNAVAILABLE",
        ClassificationFailure::ModelCannotReadImages => "MODEL_CANNOT_READ_IMAGES",
        ClassificationFailure::InvalidResponse => "INVALID_RESPONSE",
        ClassificationFailure::Timeout => "TIMEOUT",
    }
    .to_owned()
}
