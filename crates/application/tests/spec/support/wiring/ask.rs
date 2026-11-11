//! Wiring of the `ask` use cases on `TestLibrary`. The caches (`DocumentTexts`, `VectorSearch`) live in `AskKit`.

use std::sync::Arc;

use dms_application::ask::{AskLibrary, DocumentTexts, IndexLibrary, RelatedDocuments, VectorSearch};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn document_texts(&self) -> DocumentTexts {
        DocumentTexts::new(self.reader.clone(), self.folders.clone())
    }

    pub fn vector_search(&self) -> VectorSearch {
        VectorSearch::new(self.index.clone())
    }

    pub fn index_library(&self, texts: &Arc<DocumentTexts>) -> IndexLibrary {
        IndexLibrary::new(
            self.store.clone(),
            texts.clone(),
            self.embeddings.clone(),
            self.index.clone(),
            self.replicator.clone(),
        )
    }

    pub fn ask_library(&self, texts: &Arc<DocumentTexts>, vectors: &Arc<VectorSearch>) -> AskLibrary {
        AskLibrary::new(
            self.store.clone(),
            self.matcher.clone(),
            texts.clone(),
            self.embeddings.clone(),
            vectors.clone(),
            self.library_assistant.clone(),
        )
    }

    pub fn related_documents(&self, vectors: &Arc<VectorSearch>) -> RelatedDocuments {
        RelatedDocuments::new(self.store.clone(), vectors.clone())
    }
}
