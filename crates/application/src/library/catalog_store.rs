use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogMerger, CatalogUpdate};
use dms_domain::library::{LibraryFolderId, LibraryFolders};
use indexmap::IndexSet;

use crate::common::{LibrarySnapshot, OperationOutcome};
use crate::health::CatalogSnapshotKeeper;
use crate::port::state::SettingsRepository;
use crate::port::storage::{CatalogLoad, CatalogRepository, StorageFailure};
use crate::sync::{Replication, Replicator};

/// Holds the catalog of the open library and is the only writer of it.
///
/// Every change is applied to the catalog as it is at commit time, under a lock, so two use cases running at once
/// cannot overwrite each other's changes. A change is published only after the primary folder saved it.
///
/// When another device saved a newer catalog into the primary in the meantime, the primary refuses the save. The
/// store then merges that catalog into its own, publishes the merge, applies the edit once more on top and saves
/// again.
///
/// After every successful save it keeps a snapshot of the catalog in the app data directory.
pub struct CatalogStore {
    settings: Arc<dyn SettingsRepository>,
    catalogs: Arc<dyn CatalogRepository>,
    replicator: Arc<Replicator>,
    snapshots: Arc<CatalogSnapshotKeeper>,
    commit_lock: Mutex<()>,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    catalog: Option<Catalog>,
    foreign_folders: IndexSet<LibraryFolderId>,
    subscribers: Vec<Sender<Option<Catalog>>>,
}

impl CatalogStore {
    pub fn new(
        settings: Arc<dyn SettingsRepository>,
        catalogs: Arc<dyn CatalogRepository>,
        replicator: Arc<Replicator>,
        snapshots: Arc<CatalogSnapshotKeeper>,
    ) -> Self {
        Self {
            settings,
            catalogs,
            replicator,
            snapshots,
            commit_lock: Mutex::new(()),
            state: Mutex::new(State::default()),
        }
    }

    /// The catalog of the open library, or `None` while none is open.
    pub fn catalog(&self) -> Option<Catalog> {
        self.state().catalog.clone()
    }

    /// Reads the open catalog without copying it. `read` runs under the store's lock: keep it short and free of I/O.
    pub fn with_catalog<R>(&self, read: impl FnOnce(Option<&Catalog>) -> R) -> R {
        read(self.state().catalog.as_ref())
    }

    /// Every later change of [`Self::catalog`], like Kotlin's `StateFlow`: equal values are not sent again.
    /// Dropping the receiver ends the subscription.
    pub fn subscribe(&self) -> Receiver<Option<Catalog>> {
        let (sender, receiver) = channel();
        self.state().subscribers.push(sender);
        receiver
    }

    /// The folders and the catalog, or `None` while no library is open. Folders that hold another library are
    /// left out, so no use case ever writes to them. Settings that cannot be read count as no library, and so do
    /// settings that already name a foreign folder as the primary ("Make primary" saved, the library not opened
    /// again yet): every use case answers "not open" in that gap instead of failing.
    pub fn current(&self) -> Option<LibrarySnapshot> {
        let configured = self.settings.load_library_folders().ok().flatten()?;
        let state = self.state();
        let catalog = state.catalog.clone()?;
        let own =
            configured.folders().iter().filter(|folder| !state.foreign_folders.contains(&folder.id)).cloned().collect();
        // Without the primary the folders are not valid; the reopen that follows the settings change fixes it.
        let folders = LibraryFolders::new(own).ok()?;
        Some(LibrarySnapshot { folders, catalog })
    }

    /// Makes `catalog` the open catalog without saving it; used when a library is opened. `foreign` are folders
    /// holding another library, excluded until the library is opened again.
    pub fn publish(&self, catalog: Catalog, foreign: IndexSet<LibraryFolderId>) {
        let mut state = self.state();
        state.foreign_folders = foreign;
        set_catalog(&mut state, catalog);
    }

    /// Applies `edit` to the current catalog and saves the result to every library folder. `edit` may run twice:
    /// a second time on the merged catalog when another device changed the primary meanwhile. An `Err` from
    /// `edit` (Kotlin's `IllegalArgumentException`) becomes [`OperationOutcome::InvalidInput`].
    pub fn commit(&self, edit: impl Fn(&Catalog) -> Result<CatalogUpdate, DomainError>) -> OperationOutcome {
        let _guard = self.commit_lock.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(snapshot) = self.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let outcome = self.apply(&snapshot, &edit);
        if !is_catalog_changed(&outcome) {
            return outcome;
        }
        match self.merge_with_primary(snapshot) {
            Some(merged) => self.apply(&merged, &edit),
            None => outcome,
        }
    }

    fn apply(
        &self,
        snapshot: &LibrarySnapshot,
        edit: &impl Fn(&Catalog) -> Result<CatalogUpdate, DomainError>,
    ) -> OperationOutcome {
        match edit(&snapshot.catalog) {
            Err(error) => OperationOutcome::InvalidInput(error),
            Ok(CatalogUpdate::Rejected(reason)) => OperationOutcome::Rejected(reason),
            // An edit that changed nothing writes nothing.
            Ok(CatalogUpdate::Applied(catalog)) if catalog == snapshot.catalog => {
                OperationOutcome::Done { catalog, pending_folders: IndexSet::new() }
            }
            Ok(CatalogUpdate::Applied(catalog)) => self.save(snapshot, catalog),
        }
    }

    fn save(&self, snapshot: &LibrarySnapshot, catalog: Catalog) -> OperationOutcome {
        match self.replicator.save_catalog(&snapshot.folders, &catalog, snapshot.catalog.revision()) {
            Replication::PrimaryFailed(error) => OperationOutcome::StorageFailed(error),
            Replication::Done { pending_folders, .. } => {
                set_catalog(&mut self.state(), catalog.clone());
                self.snapshots.keep(&catalog);
                OperationOutcome::Done { catalog, pending_folders }
            }
        }
    }

    /// Reads the primary's catalog, merges it into the open one and publishes the result. `None` when it cannot be
    /// read or belongs to another library; the refused save then stands as the outcome.
    fn merge_with_primary(&self, snapshot: LibrarySnapshot) -> Option<LibrarySnapshot> {
        let CatalogLoad::Found { catalog: primary, .. } = self.catalogs.load(snapshot.folders.primary()) else {
            return None;
        };
        if primary.library_id() != snapshot.catalog.library_id() {
            return None;
        }
        // The primary's copy is newer, so it wins where both differ. The merge carries at least its revision, so
        // the next save is accepted unless yet another device wrote in between.
        let merged = CatalogMerger::merge(&primary, &snapshot.catalog).expect("invariant: same library").catalog;
        set_catalog(&mut self.state(), merged.clone());
        Some(LibrarySnapshot { folders: snapshot.folders, catalog: merged })
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn set_catalog(state: &mut State, catalog: Catalog) {
    if state.catalog.as_ref() == Some(&catalog) {
        return;
    }
    state.catalog = Some(catalog);
    let value = state.catalog.clone();
    state.subscribers.retain(|subscriber| subscriber.send(value.clone()).is_ok());
}

fn is_catalog_changed(outcome: &OperationOutcome) -> bool {
    matches!(outcome, OperationOutcome::StorageFailed(error) if error.reason == StorageFailure::CatalogChanged)
}
