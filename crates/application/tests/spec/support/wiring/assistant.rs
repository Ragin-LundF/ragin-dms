//! Wiring of the `assistant` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::assistant::{AskAboutDocument, DocumentReading, SuggestTags, SummarizeDocument, TagAllUntagged};

use crate::support::TestLibrary;

impl TestLibrary {
    fn reading(&self) -> Arc<DocumentReading> {
        Arc::new(DocumentReading::new(self.store.clone(), self.reader.clone()))
    }

    pub fn summarize_document(&self) -> SummarizeDocument {
        SummarizeDocument::new(self.reading(), self.assistant.clone())
    }

    pub fn suggest_tags(&self) -> SuggestTags {
        SuggestTags::new(self.reading(), self.assistant.clone(), self.tasks.clone())
    }

    pub fn tag_all_untagged(&self) -> TagAllUntagged {
        TagAllUntagged::new(self.store.clone(), self.tasks.clone(), Arc::new(self.suggest_tags()))
    }

    /// Keeps the document it read, so call `run` on one instance to see the cache.
    pub fn ask_about_document(&self) -> AskAboutDocument {
        AskAboutDocument::new(self.store.clone(), self.reading(), self.assistant.clone())
    }
}
