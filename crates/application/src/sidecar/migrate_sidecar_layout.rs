use std::sync::Arc;

use dms_domain::library::LibraryFolder;

use crate::common::Progress;
use crate::library::CatalogStore;
use crate::port::sidecar::{SidecarLayout, SidecarMove};
use crate::sidecar::{
    FolderMigration, FolderMigrationResult, KeptSidecarFile, SidecarMigrationOutcome, SidecarMigrationProgress,
};

/// Moves the flat `.dms-text/` and `.dms-index/` files of version 1.x into their subfolders, in every library folder
/// of the open library. Never replaces a file: when the subfolder already holds one for the document, both stay and
/// the flat one is reported. Safe to stop between two files and to run again; a folder is marked as done only when
/// all its files moved, and a marked folder is skipped at once. A folder that cannot be reached is moved on a later
/// run.
pub struct MigrateSidecarLayout {
    store: Arc<CatalogStore>,
    layout: Arc<dyn SidecarLayout>,
}

impl MigrateSidecarLayout {
    pub fn new(store: Arc<CatalogStore>, layout: Arc<dyn SidecarLayout>) -> Self {
        Self { store, layout }
    }

    pub fn run(&self, progress: &dyn Progress<SidecarMigrationProgress>) -> SidecarMigrationOutcome {
        let Some(library) = self.store.current() else {
            return SidecarMigrationOutcome::LibraryNotOpen;
        };
        let mut migrations = Vec::new();
        for folder in library.folders.folders() {
            if progress.is_cancelled() {
                return SidecarMigrationOutcome::Cancelled(migrations);
            }
            let (result, cancelled) = self.migrate(folder, progress);
            migrations.push(FolderMigration {
                folder_id: folder.id.clone(),
                folder_name: folder.display_name.clone(),
                result,
            });
            if cancelled {
                return SidecarMigrationOutcome::Cancelled(migrations);
            }
        }
        SidecarMigrationOutcome::Done(migrations)
    }

    /// The result for `folder`, and whether the run was cancelled in it.
    fn migrate(
        &self,
        folder: &LibraryFolder,
        progress: &dyn Progress<SidecarMigrationProgress>,
    ) -> (FolderMigrationResult, bool) {
        if self.layout.is_current(folder) {
            return (FolderMigrationResult::AlreadyCurrent, false);
        }
        let files = match self.layout.flat_files(folder) {
            Ok(files) => files,
            Err(error) => return (FolderMigrationResult::Failed(error), false),
        };
        let (mut moved, mut kept, mut failed) = (0, Vec::new(), Vec::new());
        for (index, file) in files.iter().enumerate() {
            if progress.is_cancelled() {
                return (FolderMigrationResult::Moved { moved, kept, failed, marked: false }, true);
            }
            match self.layout.move_into_subfolder(folder, file) {
                Ok(SidecarMove::Moved) => moved += 1,
                Ok(SidecarMove::KeptExisting { identical }) => {
                    kept.push(KeptSidecarFile { file: file.clone(), identical });
                }
                Err(error) => failed.push((file.clone(), error)),
            }
            progress.report(&SidecarMigrationProgress {
                folder_name: folder.display_name.clone(),
                done: index + 1,
                total: files.len(),
            });
        }
        // A marker that cannot be written only means the folder is listed again next time.
        let marked = failed.is_empty() && self.layout.mark_current(folder).is_ok();
        (FolderMigrationResult::Moved { moved, kept, failed, marked }, false)
    }
}
