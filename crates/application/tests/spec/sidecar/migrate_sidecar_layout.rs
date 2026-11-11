use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::common::{NoProgress, Progress};
use dms_application::port::sidecar::{SidecarFile, SidecarKind};
use dms_application::port::storage::{StorageError, StorageFailure};
use dms_application::sidecar::{
    FolderMigration, FolderMigrationResult, KeptSidecarFile, SidecarMigrationOutcome, SidecarMigrationProgress,
};
use dms_domain::library::LibraryFolder;
use indexmap::IndexMap;

use crate::support::TestLibrary;
use crate::support::test_data::document_id;

fn text(id: &str) -> SidecarFile {
    SidecarFile { kind: SidecarKind::Text, document_id: document_id(id) }
}

fn index(id: &str) -> SidecarFile {
    SidecarFile { kind: SidecarKind::Index, document_id: document_id(id) }
}

fn flat(library: &TestLibrary, folder: &LibraryFolder, files: &[SidecarFile]) {
    library.sidecar_layout.change(folder, |state| {
        state.flat = files.iter().map(|file| (file.clone(), format!("content of {file:?}"))).collect();
    });
}

fn folders_of(outcome: SidecarMigrationOutcome) -> Vec<FolderMigration> {
    match outcome {
        SidecarMigrationOutcome::Done(folders) => folders,
        other => panic!("expected Done, got {other:?}"),
    }
}

fn result_of(folders: &[FolderMigration], folder: &LibraryFolder) -> FolderMigrationResult {
    folders.iter().find(|migration| migration.folder_id == folder.id).expect("folder looked at").result.clone()
}

fn moved(count: usize) -> FolderMigrationResult {
    FolderMigrationResult::Moved { moved: count, kept: Vec::new(), failed: Vec::new(), marked: true }
}

/// Stops the run once `limit` files are done.
struct CancelAfter {
    limit: usize,
    done: AtomicUsize,
}

impl Progress<SidecarMigrationProgress> for CancelAfter {
    fn report(&self, _progress: &SidecarMigrationProgress) {
        self.done.fetch_add(1, Ordering::SeqCst);
    }

    fn is_cancelled(&self) -> bool {
        self.done.load(Ordering::SeqCst) >= self.limit
    }
}

#[test]
fn the_flat_files_of_every_folder_move_into_subfolders_and_each_folder_is_marked() {
    let library = TestLibrary::new();
    library.open();
    for folder in library.all_folders() {
        flat(&library, &folder, &[text("a1"), index("a1"), text("b2")]);
    }
    let reports = std::sync::Mutex::new(Vec::<(usize, usize)>::new());

    let outcome = library.migrate_sidecar_layout().run(&|step: &SidecarMigrationProgress| {
        reports.lock().expect("reports").push((step.done, step.total));
    });

    let folders = folders_of(outcome);
    for folder in library.all_folders() {
        assert_eq!(result_of(&folders, &folder), moved(3), "{}", folder.display_name);
        let state = library.sidecar_layout.folder(&folder);
        assert_eq!(state.flat, IndexMap::<SidecarFile, String>::new());
        assert_eq!(state.subfolders.keys().cloned().collect::<Vec<_>>(), vec![text("a1"), index("a1"), text("b2")]);
        assert!(state.marked);
    }
    let reports = reports.into_inner().expect("reports");
    assert_eq!(reports[..3], [(1, 3), (2, 3), (3, 3)]);
}

#[test]
fn a_marked_folder_is_skipped_without_listing_it() {
    let library = TestLibrary::new();
    library.open();
    library.migrate_sidecar_layout().run(&NoProgress);
    flat(&library, &library.primary, &[text("late")]);

    let folders = folders_of(library.migrate_sidecar_layout().run(&NoProgress));

    assert_eq!(result_of(&folders, &library.primary), FolderMigrationResult::AlreadyCurrent);
    let state = library.sidecar_layout.folder(&library.primary);
    assert_eq!(state.listings, 1);
    assert_eq!(state.flat.len(), 1);
}

#[test]
fn a_file_whose_subfolder_already_has_one_is_left_in_place_and_reported() {
    let library = TestLibrary::new();
    library.open();
    flat(&library, &library.primary, &[text("same"), text("other"), text("free")]);
    library.sidecar_layout.change(&library.primary, |state| {
        let same = state.flat[&text("same")].clone();
        state.subfolders.insert(text("same"), same);
        state.subfolders.insert(text("other"), "newer text".to_owned());
    });

    let folders = folders_of(library.migrate_sidecar_layout().run(&NoProgress));

    assert_eq!(
        result_of(&folders, &library.primary),
        FolderMigrationResult::Moved {
            moved: 1,
            kept: vec![
                KeptSidecarFile { file: text("same"), identical: true },
                KeptSidecarFile { file: text("other"), identical: false },
            ],
            failed: Vec::new(),
            marked: true,
        }
    );
    let state = library.sidecar_layout.folder(&library.primary);
    assert_eq!(state.flat.keys().cloned().collect::<Vec<_>>(), vec![text("same"), text("other")]);
    assert_eq!(state.subfolders[&text("other")], "newer text");
}

#[test]
fn a_file_that_cannot_be_moved_keeps_the_folder_unmarked_until_a_later_run_moves_it() {
    let library = TestLibrary::new();
    library.open();
    flat(&library, &library.primary, &[text("locked"), text("free")]);
    library.sidecar_layout.change(&library.primary, |state| {
        state.locked.insert(text("locked"));
    });

    let first = folders_of(library.migrate_sidecar_layout().run(&NoProgress));
    library.sidecar_layout.change(&library.primary, |state| state.locked.clear());
    let second = folders_of(library.migrate_sidecar_layout().run(&NoProgress));

    assert_eq!(
        result_of(&first, &library.primary),
        FolderMigrationResult::Moved {
            moved: 1,
            kept: Vec::new(),
            failed: vec![(text("locked"), StorageError::new(StorageFailure::IoError))],
            marked: false,
        }
    );
    assert_eq!(result_of(&second, &library.primary), moved(1));
    assert!(library.sidecar_layout.folder(&library.primary).marked);
}

#[test]
fn a_cancelled_run_stops_between_two_files_and_the_next_run_finishes() {
    let library = TestLibrary::new();
    library.open();
    flat(&library, &library.primary, &[text("a"), text("b"), text("c")]);

    let cancelled = library.migrate_sidecar_layout().run(&CancelAfter { limit: 2, done: AtomicUsize::new(0) });
    let after_cancel = library.sidecar_layout.folder(&library.primary);
    let resumed = folders_of(library.migrate_sidecar_layout().run(&NoProgress));

    let SidecarMigrationOutcome::Cancelled(folders) = cancelled else {
        panic!("expected Cancelled, got {cancelled:?}")
    };
    assert_eq!(
        result_of(&folders, &library.primary),
        FolderMigrationResult::Moved { moved: 2, kept: Vec::new(), failed: Vec::new(), marked: false }
    );
    assert_eq!(folders.len(), 1, "the sync folders are not looked at after the cancel");
    assert_eq!(after_cancel.flat.keys().cloned().collect::<Vec<_>>(), vec![text("c")]);
    assert!(!after_cancel.marked);
    assert_eq!(result_of(&resumed, &library.primary), moved(1));
    assert_eq!(library.sidecar_layout.folder(&library.primary).subfolders.len(), 3);
}

#[test]
fn an_offline_folder_is_moved_once_it_is_reachable_again() {
    let library = TestLibrary::new();
    library.open();
    flat(&library, &library.sync_b, &[index("x")]);
    library.folders.make_unreachable(&library.sync_b);

    let offline = folders_of(library.migrate_sidecar_layout().run(&NoProgress));
    library.folders.make_reachable(&library.sync_b);
    let online = folders_of(library.migrate_sidecar_layout().run(&NoProgress));

    assert_eq!(
        result_of(&offline, &library.sync_b),
        FolderMigrationResult::Failed(StorageError::new(StorageFailure::FolderUnreachable))
    );
    assert_eq!(result_of(&offline, &library.primary), moved(0));
    assert_eq!(result_of(&online, &library.primary), FolderMigrationResult::AlreadyCurrent);
    assert_eq!(result_of(&online, &library.sync_b), moved(1));
}

#[test]
fn nothing_is_moved_while_no_library_is_open() {
    let library = TestLibrary::new();
    flat(&library, &library.primary, &[text("a")]);

    let outcome = library.migrate_sidecar_layout().run(&NoProgress);

    assert_eq!(outcome, SidecarMigrationOutcome::LibraryNotOpen);
    assert_eq!(library.sidecar_layout.folder(&library.primary).flat.len(), 1);
}
