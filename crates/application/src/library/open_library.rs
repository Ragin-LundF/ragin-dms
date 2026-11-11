use std::sync::Arc;

use dms_domain::catalog::{Catalog, CatalogMergeResult, CatalogMerger, CatalogRevision, LibraryId};
use dms_domain::library::{LibraryFolder, LibraryFolderId, LibraryFolders};
use indexmap::IndexSet;

use crate::library::{CatalogStore, OpenLibraryOutcome};
use crate::port::state::{SettingsRepository, TaskStore};
use crate::port::storage::{CatalogLoad, CatalogRepository};
use crate::port::system::IdGenerator;
use crate::sync::{Replication, Replicator};

/// Opens the library: reads the catalog of every reachable folder, merges all copies (including conflict copies
/// created by sync clients), saves the merged catalog where it is missing or differs, and publishes it.
///
/// A library without any catalog starts empty. A folder holding another library's catalog is ignored and
/// reported, never overwritten.
pub struct OpenLibrary {
    settings: Arc<dyn SettingsRepository>,
    catalogs: Arc<dyn CatalogRepository>,
    replicator: Arc<Replicator>,
    store: Arc<CatalogStore>,
    tasks: Arc<dyn TaskStore>,
    ids: Arc<dyn IdGenerator>,
}

/// A folder whose catalog file was found.
struct Found<'a> {
    folder: &'a LibraryFolder,
    catalog: &'a Catalog,
    conflict_copies: &'a [Catalog],
}

impl OpenLibrary {
    pub fn new(
        settings: Arc<dyn SettingsRepository>,
        catalogs: Arc<dyn CatalogRepository>,
        replicator: Arc<Replicator>,
        store: Arc<CatalogStore>,
        tasks: Arc<dyn TaskStore>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { settings, catalogs, replicator, store, tasks, ids }
    }

    pub fn run(&self) -> OpenLibraryOutcome {
        let Some(folders) = self.settings.load_library_folders().ok().flatten() else {
            return OpenLibraryOutcome::NotConfigured;
        };
        let loads: Vec<(&LibraryFolder, CatalogLoad)> =
            folders.folders().iter().map(|folder| (folder, self.catalogs.load(folder))).collect();
        if let Some((_, CatalogLoad::Failed(error))) = loads.iter().find(|(folder, _)| *folder == folders.primary()) {
            return OpenLibraryOutcome::PrimaryFailed(error.clone());
        }
        let found: Vec<Found<'_>> = loads
            .iter()
            .filter_map(|(folder, load)| match load {
                CatalogLoad::Found { catalog, conflict_copies } => Some(Found { folder, catalog, conflict_copies }),
                _ => None,
            })
            .collect();
        let library_id = self.library_id_of(&folders, &found);
        let foreign: IndexSet<LibraryFolderId> = found
            .iter()
            .filter(|found| found.catalog.library_id() != &library_id)
            .map(|found| found.folder.id.clone())
            .collect();
        let own: Vec<&Found<'_>> = found.iter().filter(|found| !foreign.contains(&found.folder.id)).collect();
        // A conflict copy of another library's catalog cannot be merged; it stays where it is.
        let copies: Vec<&Catalog> = own
            .iter()
            .flat_map(|found| {
                std::iter::once(found.catalog)
                    .chain(found.conflict_copies.iter().filter(|copy| copy.library_id() == &library_id))
            })
            .collect();
        let merge = merge_all(&library_id, &copies);
        let newest_seen = copies.iter().map(|copy| copy.revision()).max().unwrap_or(CatalogRevision::INITIAL);

        let any_outdated =
            loads.iter().any(|(folder, load)| !foreign.contains(&folder.id) && is_outdated(load, &merge.catalog));
        let pending = if any_outdated {
            // The primary's id wins, so a foreign folder is always a sync folder and can simply be left out.
            let own_folders =
                folders.folders().iter().filter(|folder| !foreign.contains(&folder.id)).cloned().collect();
            let own_folders = LibraryFolders::new(own_folders).expect("invariant: the primary is never foreign");
            match self.replicator.save_catalog(&own_folders, &merge.catalog, newest_seen) {
                Replication::PrimaryFailed(error) => return OpenLibraryOutcome::PrimaryFailed(error),
                Replication::Done { pending_folders, .. } => pending_folders,
            }
        } else {
            IndexSet::new()
        };
        self.discard_merged_conflict_copies(&own, &pending);

        self.store.publish(merge.catalog.clone(), foreign.clone());
        // Best effort, like the activity log: the library is open either way.
        let _ = self.tasks.replace_merge_conflicts(&merge.conflicts);
        OpenLibraryOutcome::Opened {
            catalog: merge.catalog,
            merge_conflicts: merge.conflicts,
            unreachable_folders: loads
                .iter()
                .filter(|(_, load)| matches!(load, CatalogLoad::Failed(_)))
                .map(|(folder, _)| folder.id.clone())
                .collect(),
            foreign_folders: foreign,
            pending_folders: pending,
        }
    }

    /// The primary's library id wins; without any catalog a new library begins.
    fn library_id_of(&self, folders: &LibraryFolders, found: &[Found<'_>]) -> LibraryId {
        let catalog = found
            .iter()
            .find(|found| found.folder == folders.primary())
            .or_else(|| found.first())
            .map(|found| found.catalog);
        match catalog {
            Some(catalog) => catalog.library_id().clone(),
            None => LibraryId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
        }
    }

    fn discard_merged_conflict_copies(&self, own: &[&Found<'_>], pending: &IndexSet<LibraryFolderId>) {
        own.iter().filter(|found| !found.conflict_copies.is_empty() && !pending.contains(&found.folder.id)).for_each(
            |found| {
                let _ = self.catalogs.discard_conflict_copies(found.folder);
            },
        );
    }
}

/// Unreachable folders are left alone; they get the catalog with the next sync or retry.
fn is_outdated(load: &CatalogLoad, merged: &Catalog) -> bool {
    match load {
        CatalogLoad::NotFound => true,
        CatalogLoad::Found { catalog, conflict_copies } => catalog != merged || !conflict_copies.is_empty(),
        CatalogLoad::Failed(_) => false,
    }
}

fn merge_all(library_id: &LibraryId, copies: &[&Catalog]) -> CatalogMergeResult {
    let first = copies.first().map_or_else(|| Catalog::empty(library_id.clone()), |first| (*first).clone());
    let initial = CatalogMergeResult { catalog: first, conflicts: Vec::new() };
    copies.iter().skip(1).fold(initial, |mut merged, copy| {
        let result = CatalogMerger::merge(&merged.catalog, copy).expect("invariant: copies of one library");
        merged.conflicts.extend(result.conflicts);
        CatalogMergeResult { catalog: result.catalog, conflicts: merged.conflicts }
    })
}
