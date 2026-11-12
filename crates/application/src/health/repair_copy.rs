use std::sync::Arc;

use dms_domain::document::{Document, DocumentId};
use dms_domain::health::HealthFinding;
use dms_domain::library::{LibraryFolder, LibraryFolderId};

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::health::HealthStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation, StorageResult};
use crate::sync::Replicator;

/// Replaces the damaged copy of a document in one folder with the intact copy of another: the damaged file goes
/// to the trash of its folder, the intact one is copied in, verified by hash. The source is compared with the
/// catalog right before, so a copy that changed meanwhile never spreads. A repair from the primary into an offline
/// sync folder waits in the journal. Requires the user's confirmation.
pub struct RepairCopy {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    health: Arc<dyn HealthStore>,
    activity: Arc<ActivityRecorder>,
}

impl RepairCopy {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        health: Arc<dyn HealthStore>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, health, activity }
    }

    pub fn run(
        &self,
        document_id: &DocumentId,
        damaged_folder_id: &LibraryFolderId,
        source_folder_id: &LibraryFolderId,
        confirmed: bool,
    ) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let folders = &library.folders;
        let (Some(document), Some(damaged), Some(source)) = (
            library.catalog.documents().get(document_id),
            folders.find(damaged_folder_id),
            folders.find(source_folder_id),
        ) else {
            return OperationOutcome::NotFound;
        };
        if damaged == source {
            return OperationOutcome::NotFound;
        }
        match self.matches_catalog(source, document) {
            Ok(true) => {}
            Ok(false) => return OperationOutcome::SourceChanged,
            Err(error) => return OperationOutcome::StorageFailed(error),
        }
        let pending = if matches!(self.matches_catalog(damaged, document), Ok(true)) {
            // Repaired meanwhile, by a sync client or a second click.
            std::iter::empty().collect()
        } else if source == folders.primary() {
            self.replicator.distribute(
                folders,
                &[damaged],
                &FolderOperation::CopyFromPrimary { path: document.path.clone(), replace_existing: true },
            )
        } else {
            if let Err(error) = self.storage.copy_file(source, damaged, &document.path, true) {
                return OperationOutcome::StorageFailed(error);
            }
            std::iter::empty().collect()
        };
        if !pending.contains(damaged_folder_id) {
            self.activity.record(ActivityKind::DocumentRestored, document.path.value());
            self.forget_finding(document_id, damaged_folder_id);
        }
        OperationOutcome::Done { catalog: library.catalog.clone(), pending_folders: pending }
    }

    fn matches_catalog(&self, folder: &LibraryFolder, document: &Document) -> StorageResult<bool> {
        let facts = self.storage.read_facts(folder, &document.path)?;
        Ok(facts.size == document.size && facts.content_hash.as_ref() == Some(&document.content_hash))
    }

    /// Best effort: the next check finds the copy intact anyway.
    fn forget_finding(&self, document_id: &DocumentId, folder: &LibraryFolderId) {
        let Ok(Some(mut report)) = self.health.load() else { return };
        report.findings.retain(|finding| {
            !matches!(finding, HealthFinding::Damaged { document_id: id, folder_id, .. }
                if id == document_id && folder_id == folder)
        });
        let _ = self.health.save(&report);
    }
}
