//! Wiring of the `classification` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::classification::{ApproveSuggestion, ClassifyAllNew, RejectSuggestion, SuggestClassification};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn suggest_classification(&self) -> SuggestClassification {
        SuggestClassification::new(
            self.store.clone(),
            self.reader.clone(),
            self.classifier.clone(),
            self.tasks.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.clock.clone(),
        )
    }

    pub fn approve_suggestion(&self) -> ApproveSuggestion {
        ApproveSuggestion::new(
            self.store.clone(),
            self.tasks.clone(),
            Arc::new(self.filing()),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }

    pub fn reject_suggestion(&self) -> RejectSuggestion {
        RejectSuggestion::new(self.tasks.clone())
    }

    pub fn classify_all_new(&self) -> ClassifyAllNew {
        ClassifyAllNew::new(self.store.clone(), self.tasks.clone(), Arc::new(self.suggest_classification()))
    }
}
