use std::sync::Arc;

use dms_domain::change::{ChangeDetector, DetectedChange, FolderSnapshot, MovedFile};
use dms_domain::library::{LibraryFolder, RelativePath};
use dms_domain::text;
use indexmap::{IndexMap, IndexSet};

use crate::common::{ActivityRecorder, LibrarySnapshot};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{FolderOperation, LibraryScanner, NoScanListener};
use crate::sync::{FileComparison, Replicator, SyncAction, SyncActionKind, SyncOutcome, SyncReport};

/// "Sync to all destinations": makes every reachable sync folder contain what the primary contains.
///
/// Applies moves the catalog knows (a document still under its old path in the sync folder), copies files the sync
/// folder lacks, and saves the catalog there. Never deletes and never overwrites: files only in the sync folder are
/// left alone, files with different content are reported as conflicts.
pub struct SyncFromPrimary {
    store: Arc<CatalogStore>,
    scanner: Arc<dyn LibraryScanner>,
    replicator: Arc<Replicator>,
    activity: Arc<ActivityRecorder>,
}

impl SyncFromPrimary {
    pub fn new(
        store: Arc<CatalogStore>,
        scanner: Arc<dyn LibraryScanner>,
        replicator: Arc<Replicator>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, scanner, replicator, activity }
    }

    pub fn run(&self) -> SyncOutcome {
        let Some(library) = self.store.current() else {
            return SyncOutcome::LibraryNotOpen;
        };
        let source = match self.scanner.scan(library.folders.primary(), &library.catalog, &NoScanListener) {
            Ok(source) => source,
            Err(error) => return SyncOutcome::PrimaryFailed(error),
        };
        let report = library
            .folders
            .sync_folders()
            .into_iter()
            .map(|target| self.sync_folder(&library, &source, target))
            .fold(SyncReport::default(), |total, report| total + report);
        self.activity.record(ActivityKind::Synchronized, &library.folders.primary().display_name);
        SyncOutcome::Completed(report)
    }

    fn sync_folder(&self, library: &LibrarySnapshot, source: &FolderSnapshot, target: &LibraryFolder) -> SyncReport {
        let Ok(snapshot) = self.scanner.scan(target, &library.catalog, &NoScanListener) else {
            return SyncReport { unreachable_folders: IndexSet::from([target.id.clone()]), ..SyncReport::default() };
        };
        let mut target_files = snapshot.files.clone();
        let moves: Vec<SyncReport> = known_moves(library, source, &snapshot)
            .iter()
            .map(|moved| {
                let report = self.move_in_target(library, target, moved);
                if report.pending_folders.is_empty()
                    && let Some(facts) = target_files.shift_remove(&moved.path)
                {
                    target_files.insert(moved.previous_path.clone(), facts);
                }
                report
            })
            .collect();
        let copies: Vec<SyncReport> = sorted(source.files.keys().filter(|path| !target_files.contains_key(*path)))
            .into_iter()
            .map(|path| self.copy_to_target(library, target, path))
            .collect();
        let conflicts = sorted(
            source
                .files
                .iter()
                .filter(|(path, facts)| {
                    target_files
                        .get(*path)
                        .is_some_and(|other| FileComparison::differs(facts, other, path, &library.catalog))
                })
                .map(|(path, _)| path),
        );
        let extra = sorted(target_files.keys().filter(|path| !source.files.contains_key(*path)));
        let catalog_pending = self.replicator.distribute(&library.folders, &[target], &FolderOperation::SaveCatalog);
        let remaining = SyncReport {
            conflicts: conflicts.into_iter().cloned().collect(),
            left_alone: if extra.is_empty() {
                IndexMap::new()
            } else {
                IndexMap::from([(target.id.clone(), extra.into_iter().cloned().collect())])
            },
            pending_folders: catalog_pending,
            ..SyncReport::default()
        };
        moves.into_iter().chain(copies).fold(remaining, |total, report| total + report)
    }

    fn move_in_target(&self, library: &LibrarySnapshot, target: &LibraryFolder, moved: &MovedFile) -> SyncReport {
        let operation = FolderOperation::Move { from: moved.path.clone(), to: moved.previous_path.clone() };
        self.apply(library, target, &operation, SyncActionKind::Moved, &moved.previous_path)
    }

    fn copy_to_target(&self, library: &LibrarySnapshot, target: &LibraryFolder, path: &RelativePath) -> SyncReport {
        let operation = FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: false };
        self.apply(library, target, &operation, SyncActionKind::Copied, path)
    }

    fn apply(
        &self,
        library: &LibrarySnapshot,
        target: &LibraryFolder,
        operation: &FolderOperation,
        kind: SyncActionKind,
        path: &RelativePath,
    ) -> SyncReport {
        let pending = self.replicator.distribute(&library.folders, &[target], operation);
        if !pending.is_empty() {
            return SyncReport { pending_folders: pending, ..SyncReport::default() };
        }
        let action = SyncAction { folder_id: target.id.clone(), kind, path: path.clone() };
        SyncReport { actions: vec![action], ..SyncReport::default() }
    }
}

/// Documents the sync folder still holds under a path the primary no longer uses.
fn known_moves(library: &LibrarySnapshot, source: &FolderSnapshot, snapshot: &FolderSnapshot) -> Vec<MovedFile> {
    ChangeDetector::detect(&library.catalog, std::slice::from_ref(snapshot))
        .into_iter()
        .filter_map(|change| match change {
            DetectedChange::MovedFile(moved) if source.files.contains_key(&moved.previous_path) => Some(moved),
            _ => None,
        })
        .collect()
}

fn sorted<'a>(paths: impl Iterator<Item = &'a RelativePath>) -> Vec<&'a RelativePath> {
    let mut paths: Vec<&RelativePath> = paths.collect();
    paths.sort_by(|first, second| text::compare(first.value(), second.value()));
    paths
}
