use std::sync::Arc;

use dms_domain::catalog::CatalogUpdate;
use dms_domain::settings::{InboxSettings, LibraryLayout};
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::sync::{Replication, Replicator};

/// Saves the inbox settings to the catalog, so every device uses them. Their path was checked when they were
/// created (see `InboxSettings::path_violation`). Switching the inbox on creates its folder in every library
/// folder first; an existing folder is kept as it is.
pub struct ConfigureInbox {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    activity: Arc<ActivityRecorder>,
}

impl ConfigureInbox {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, activity }
    }

    pub fn run(&self, inbox: &InboxSettings) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let mut pending = IndexSet::new();
        if inbox.enabled {
            let created = self.replicator.replicate(
                &library.folders,
                |primary| self.storage.create_folder(primary, &inbox.path),
                &FolderOperation::CreateFolder(inbox.path.clone()),
            );
            match created {
                Replication::PrimaryFailed(error) => return OperationOutcome::StorageFailed(error),
                Replication::Done { pending_folders, .. } => pending = pending_folders,
            }
        }
        let outcome = self
            .store
            .commit(|catalog| {
                let layout = LibraryLayout { inbox: inbox.clone(), ..catalog.settings().layout.clone() };
                if layout == catalog.settings().layout {
                    Ok(CatalogUpdate::Applied(catalog.clone()))
                } else {
                    catalog.change_settings(catalog.settings().with_layout(layout))
                }
            })
            .with_pending_folders(&pending);
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::CatalogEdited, &format!("Inbox {}", inbox.path));
        }
        outcome
    }
}
