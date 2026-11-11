use std::sync::Arc;

use dms_domain::change::DetectedChange;
use indexmap::IndexSet;

use crate::common::OperationOutcome;
use crate::document::AdoptNewFile;
use crate::inbox::CollectInboxOutcome;
use crate::library::{CatalogStore, ScanResult};

/// Catalogs the new files a scan found in the inbox of any library folder. A file that arrived in a sync folder is
/// copied into the primary first, verified, and from there into the other folders (see [`AdoptNewFile`]). Copies
/// of a catalogued document are left to the dashboard.
pub struct CollectInbox {
    store: Arc<CatalogStore>,
    adopt_new_file: Arc<AdoptNewFile>,
}

impl CollectInbox {
    pub fn new(store: Arc<CatalogStore>, adopt_new_file: Arc<AdoptNewFile>) -> Self {
        Self { store, adopt_new_file }
    }

    pub fn run(&self, scan: &ScanResult) -> CollectInboxOutcome {
        let Some(library) = self.store.current() else {
            return CollectInboxOutcome::LibraryNotOpen;
        };
        let inbox = &library.catalog.settings().layout.inbox;
        if !inbox.enabled {
            return CollectInboxOutcome::Collected(Vec::new());
        }
        let known: IndexSet<_> = library.catalog.documents().values().map(|it| &it.content_hash).collect();
        let mut seen = IndexSet::new();
        let candidates: Vec<_> = scan
            .changes
            .iter()
            .filter_map(|change| match change {
                DetectedChange::NewFile(new_file) => Some(new_file),
                _ => None,
            })
            .filter(|it| inbox.contains(&it.path) && it.content_hash.as_ref().is_some_and(|hash| !known.contains(hash)))
            .filter(|it| seen.insert(it.content_hash.clone()))
            .collect();
        let collected = candidates
            .into_iter()
            .filter(|it| matches!(self.adopt_new_file.run(it), OperationOutcome::Done { .. }))
            .cloned()
            .collect();
        CollectInboxOutcome::Collected(collected)
    }
}
