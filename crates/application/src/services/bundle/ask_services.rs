use std::sync::Arc;

use crate::ask::{AskLibrary, IndexLibrary, RelatedDocuments};

/// Asking the library: building the embedding index, answering questions, and related documents.
pub struct AskServices {
    pub index_library: Arc<IndexLibrary>,
    pub ask_library: Arc<AskLibrary>,
    pub related_documents: Arc<RelatedDocuments>,
}
