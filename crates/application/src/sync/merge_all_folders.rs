use std::sync::Arc;

use dms_domain::change::{ChangeDetector, DetectedChange, FolderSnapshot, MovedFile};
use dms_domain::library::{LibraryFolder, LibraryFolderId, RelativePath};
use dms_domain::text;
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, LibrarySnapshot};
use crate::library::{CatalogStore, OpenLibrary, OpenLibraryOutcome};
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation, LibraryScanner, NoScanListener};
use crate::sync::{FileComparison, Replicator, SyncAction, SyncActionKind, SyncOutcome, SyncReport};

/// "Merge all folders": scans every reachable folder, including backups, and copies each file that is missing
/// somewhere into the folders that lack it, then merges and saves the catalogs.
///
/// Never deletes and never overwrites. A path with different content in different folders is a conflict for the
/// user. A document moved outside the tool is left alone until the user confirms or undoes that move: neither its
/// new nor its old path is copied, so a move never becomes a duplicate.
pub struct MergeAllFolders {
    store: Arc<CatalogStore>,
    scanner: Arc<dyn LibraryScanner>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    open_library: Arc<OpenLibrary>,
    activity: Arc<ActivityRecorder>,
}

impl MergeAllFolders {
    pub fn new(
        store: Arc<CatalogStore>,
        scanner: Arc<dyn LibraryScanner>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        open_library: Arc<OpenLibrary>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, scanner, storage, replicator, open_library, activity }
    }

    pub fn run(&self) -> SyncOutcome {
        let Some(library) = self.store.current() else {
            return SyncOutcome::LibraryNotOpen;
        };
        let mut snapshots: Vec<(&LibraryFolder, FolderSnapshot)> = Vec::new();
        let mut unreachable: IndexSet<LibraryFolderId> = IndexSet::new();
        for folder in library.folders.folders() {
            match self.scanner.scan(folder, &library.catalog, &NoScanListener) {
                Ok(snapshot) => snapshots.push((folder, snapshot)),
                Err(error) if folder == library.folders.primary() => return SyncOutcome::PrimaryFailed(error),
                Err(_) => {
                    unreachable.insert(folder.id.clone());
                }
            }
        }
        let all: Vec<FolderSnapshot> = snapshots.iter().map(|(_, snapshot)| snapshot.clone()).collect();
        let moves: Vec<MovedFile> = ChangeDetector::detect(&library.catalog, &all)
            .into_iter()
            .filter_map(|change| match change {
                DetectedChange::MovedFile(moved) => Some(moved),
                _ => None,
            })
            .collect();
        let mut paths: Vec<&RelativePath> = snapshots
            .iter()
            .flat_map(|(_, snapshot)| snapshot.files.keys())
            .collect::<IndexSet<_>>()
            .into_iter()
            .collect();
        paths.sort_by(|first, second| text::compare(first.value(), second.value()));
        let files: Vec<SyncReport> =
            paths.iter().map(|path| self.merge_path(&library, &snapshots, path, &moves)).collect();
        let opened = self.open_library.run();
        self.activity.record(ActivityKind::Synchronized, &library.folders.primary().display_name);
        let start = SyncReport { unreachable_folders: unreachable, ..SyncReport::default() };
        let total = files.into_iter().fold(start, |sum, report| sum + report);
        match opened {
            OpenLibraryOutcome::Opened { pending_folders, .. } => {
                SyncOutcome::Completed(total + SyncReport { pending_folders, ..SyncReport::default() })
            }
            OpenLibraryOutcome::PrimaryFailed(error) => SyncOutcome::PrimaryFailed(error),
            OpenLibraryOutcome::NotConfigured => SyncOutcome::LibraryNotOpen,
        }
    }

    fn merge_path(
        &self,
        library: &LibrarySnapshot,
        snapshots: &[(&LibraryFolder, FolderSnapshot)],
        path: &RelativePath,
        moves: &[MovedFile],
    ) -> SyncReport {
        let holders: Vec<&(&LibraryFolder, FolderSnapshot)> =
            snapshots.iter().filter(|(_, snapshot)| snapshot.files.contains_key(path)).collect();
        // A folder where the user moved a document away from this path does not get it back.
        let moved_away: IndexSet<&LibraryFolderId> =
            moves.iter().filter(|moved| &moved.previous_path == path).map(|moved| &moved.folder_id).collect();
        let lacking: Vec<&LibraryFolder> = snapshots
            .iter()
            .map(|(folder, _)| *folder)
            .filter(|folder| !holders.iter().any(|(holder, _)| holder == folder) && !moved_away.contains(&folder.id))
            .collect();
        if lacking.is_empty() {
            return SyncReport::default();
        }
        if moves.iter().any(|moved| &moved.path == path) {
            let left_alone = holders.iter().map(|(folder, _)| (folder.id.clone(), vec![path.clone()])).collect();
            return SyncReport { left_alone, ..SyncReport::default() };
        }
        let hashes: Vec<_> = holders
            .iter()
            .map(|(_, snapshot)| FileComparison::effective_hash(&snapshot.files[path], path, &library.catalog))
            .collect();
        if hashes.iter().any(Option::is_none) || hashes.iter().collect::<IndexSet<_>>().len() > 1 {
            return SyncReport { conflicts: vec![path.clone()], ..SyncReport::default() };
        }
        self.copy_to_lacking(library, path, holders[0].0, &lacking)
    }

    /// Copies into the primary first if it lacks the file, then from the primary into the sync folders.
    fn copy_to_lacking(
        &self,
        library: &LibrarySnapshot,
        path: &RelativePath,
        source: &LibraryFolder,
        lacking: &[&LibraryFolder],
    ) -> SyncReport {
        let primary = library.folders.primary();
        let mut actions = Vec::new();
        if lacking.contains(&primary) {
            if self.storage.copy_file(source, primary, path, false).is_err() {
                return SyncReport { failed_paths: vec![path.clone()], ..SyncReport::default() };
            }
            actions.push(SyncAction {
                folder_id: primary.id.clone(),
                kind: SyncActionKind::Copied,
                path: path.clone(),
            });
        }
        let targets: Vec<&LibraryFolder> =
            library.folders.sync_folders().into_iter().filter(|folder| lacking.contains(folder)).collect();
        let pending = self.replicator.distribute(
            &library.folders,
            &targets,
            &FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: false },
        );
        actions.extend(targets.iter().filter(|folder| !pending.contains(&folder.id)).map(|folder| SyncAction {
            folder_id: folder.id.clone(),
            kind: SyncActionKind::Copied,
            path: path.clone(),
        }));
        SyncReport { actions, pending_folders: pending, ..SyncReport::default() }
    }
}
