use std::sync::Arc;

use dms_domain::text;
use indexmap::IndexSet;

use crate::assistant::SuggestTags;
use crate::classification::ClassificationProgress;
use crate::common::Progress;
use crate::library::CatalogStore;
use crate::port::state::TaskStore;

/// Lets the AI suggest tags for every document it can read that has no tags and no open suggestion, one after the
/// other. Each result is reported, so the UI can show progress and stop the batch.
pub struct TagAllUntagged {
    store: Arc<CatalogStore>,
    tasks: Arc<dyn TaskStore>,
    suggest_tags: Arc<SuggestTags>,
}

impl TagAllUntagged {
    pub fn new(store: Arc<CatalogStore>, tasks: Arc<dyn TaskStore>, suggest_tags: Arc<SuggestTags>) -> Self {
        Self { store, tasks, suggest_tags }
    }

    /// Reports each document's outcome and returns the last report; `None` when nothing was tagged.
    pub fn run(&self, progress: &dyn Progress<ClassificationProgress>) -> Option<ClassificationProgress> {
        let library = self.store.current()?;
        // A task store that cannot be read has no pending suggestions to skip.
        let pending: IndexSet<_> =
            self.tasks.suggestions().unwrap_or_default().into_iter().map(|it| it.document_id).collect();
        let mut candidates: Vec<_> = library
            .catalog
            .documents()
            .values()
            .filter(|it| it.tags.is_empty() && it.kind().readable_by_ai() && !pending.contains(&it.id))
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
                outcome: self.suggest_tags.run(&document.id),
            };
            progress.report(&step);
            last = Some(step);
        }
        last
    }
}
