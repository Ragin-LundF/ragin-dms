use std::sync::Arc;

use dms_application::ask::{AskLibrary, AskOutcome, DocumentTexts, IndexProgress, RelatedDocuments, VectorSearch};
use dms_application::common::Progress;
use dms_domain::query::DocumentQuery;

use crate::support::Shared;
use crate::support::TestLibrary;

/// A `TestLibrary` with the ask caches, which the Kotlin `TestLibrary` created once.
pub struct AskKit {
    pub library: TestLibrary,
    texts: Arc<DocumentTexts>,
    vectors: Arc<VectorSearch>,
}

impl AskKit {
    pub fn new(library: TestLibrary) -> Self {
        let texts = Arc::new(library.document_texts());
        let vectors = Arc::new(library.vector_search());
        Self { library, texts, vectors }
    }

    /// Every progress the index run reported (Kotlin's `indexLibrary().toList()`).
    pub fn index(&self) -> Vec<IndexProgress> {
        let reported = Shared::new(Vec::new());
        let collect = |step: &IndexProgress| reported.update(|all| all.push(step.clone()));
        self.library.index_library(&self.texts).run(&collect as &dyn Progress<IndexProgress>);
        reported.get()
    }

    pub fn ask(&self, question: &str, scope: &DocumentQuery) -> AskOutcome {
        self.ask_library().run(question, scope)
    }

    pub fn ask_library(&self) -> AskLibrary {
        self.library.ask_library(&self.texts, &self.vectors)
    }

    pub fn related_documents(&self) -> RelatedDocuments {
        self.library.related_documents(&self.vectors)
    }
}
