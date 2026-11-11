use dms_domain::DomainError;
use dms_domain::document::{ContentHash, DocumentId};

use crate::port::index::IndexedChunk;

/// The embeddings of one document, made by [`Self::model`] with [`Self::dimensions`] values per vector, for the
/// file with [`Self::content_hash`] and the metadata with [`Self::metadata_hash`]. A stale hash or another model
/// means the entry is rebuilt; it is derived data and never merged.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentVectors {
    pub document_id: DocumentId,
    pub model: String,
    pub dimensions: usize,
    pub content_hash: ContentHash,
    pub metadata_hash: String,
    pub chunks: Vec<IndexedChunk>,
}

impl DocumentVectors {
    /// # Errors
    /// Fails when a vector does not have [`Self::dimensions`] values.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.chunks.iter().all(|chunk| chunk.vector.len() == self.dimensions) {
            Ok(())
        } else {
            Err(DomainError::InvalidArgument(format!("Every vector needs {} values", self.dimensions)))
        }
    }

    #[must_use]
    pub fn is_current(&self, model: &str, content_hash: &ContentHash, metadata_hash: &str) -> bool {
        self.model == model && &self.content_hash == content_hash && self.metadata_hash == metadata_hash
    }
}
