use std::sync::Arc;

use dms_domain::library::RelativePath;
use dms_domain::text;
use indexmap::IndexSet;

use crate::classification::{ClassificationProgress, SuggestClassification};
use crate::common::Progress;
use crate::library::CatalogStore;
use crate::port::state::TaskStore;

/// Suggests a classification for every document the AI can read that has neither a confirmed classification nor
/// a pending suggestion, one after the other, optionally only below one folder such as the inbox. Cancelling
/// stops it before the next document.
pub struct ClassifyAllNew {
    store: Arc<CatalogStore>,
    tasks: Arc<dyn TaskStore>,
    suggest: Arc<SuggestClassification>,
}

impl ClassifyAllNew {
    pub fn new(store: Arc<CatalogStore>, tasks: Arc<dyn TaskStore>, suggest: Arc<SuggestClassification>) -> Self {
        Self { store, tasks, suggest }
    }

    /// Reports each document's outcome and returns the last report; `None` when nothing was classified.
    pub fn run(
        &self,
        scope: &RelativePath,
        progress: &dyn Progress<ClassificationProgress>,
    ) -> Option<ClassificationProgress> {
        let library = self.store.current()?;
        // A task store that cannot be read has no pending suggestions to skip.
        let pending: IndexSet<_> =
            self.tasks.suggestions().unwrap_or_default().into_iter().map(|it| it.document_id).collect();
        let mut candidates: Vec<_> = library
            .catalog
            .documents()
            .values()
            .filter(|it| it.classification.is_none() && it.kind().readable_by_ai() && !pending.contains(&it.id))
            .filter(|it| it.path.is_inside(scope))
            .collect();
        candidates.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
        let mut last = None;
        for (index, document) in candidates.iter().enumerate() {
            if progress.is_cancelled() {
                break;
            }
            let step = ClassificationProgress {
                done: index + 1,
                total: candidates.len(),
                document_id: document.id.clone(),
                outcome: self.suggest.run(&document.id),
            };
            progress.report(&step);
            last = Some(step);
        }
        last
    }
}
