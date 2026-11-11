use std::collections::HashSet;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use dms_domain::change::{ChangeDetector, FileFacts, FolderSnapshot};
use dms_domain::library::{FolderRole, LibraryFolder, LibraryFolderId, RelativePath};
use indexmap::{IndexMap, IndexSet};

use crate::common::Progress;
use crate::library::{AddScannedFiles, CatalogStore, ScanOutcome, ScanProgress, ScanResult, ScannedFile};
use crate::port::storage::{LibraryScanner, ScanListener, StorageError, StorageFailure, StorageResult};

const ADD_INTERVAL: Duration = Duration::from_secs(2);

/// Scans every library folder and reports how they differ from the catalog.
///
/// With `add_new_files`, files of the primary folder that the catalog does not know are catalogued while the scan
/// reads them (see [`AddScannedFiles`]), so they show up in the library long before the scan ends. Otherwise the
/// scan changes nothing.
///
/// A cancelled scan (`progress.is_cancelled()`) stops the folder being read, still catalogues what it found so far
/// (a catalog write is never cut in half), and returns [`ScanOutcome::Cancelled`].
pub struct ScanLibrary {
    store: Arc<CatalogStore>,
    scanner: Arc<dyn LibraryScanner>,
    add_scanned_files: Arc<AddScannedFiles>,
}

/// Why a scan ended without a result.
enum Stopped {
    LibraryClosed,
    Cancelled,
}

impl ScanLibrary {
    pub fn new(
        store: Arc<CatalogStore>,
        scanner: Arc<dyn LibraryScanner>,
        add_scanned_files: Arc<AddScannedFiles>,
    ) -> Self {
        Self { store, scanner, add_scanned_files }
    }

    /// `progress` follows the scan file by file; it is called on the scanning thread.
    pub fn run(&self, add_new_files: bool, progress: &dyn Progress<ScanProgress>) -> ScanOutcome {
        match self.scan_all(add_new_files, progress) {
            Ok(result) => ScanOutcome::Scanned(result),
            Err(Stopped::LibraryClosed) => ScanOutcome::LibraryNotOpen,
            Err(Stopped::Cancelled) => ScanOutcome::Cancelled,
        }
    }

    fn scan_all(&self, add_new_files: bool, progress: &dyn Progress<ScanProgress>) -> Result<ScanResult, Stopped> {
        let folders = self.store.current().ok_or(Stopped::LibraryClosed)?.folders;
        let folders = folders.folders();
        let mut snapshots: Vec<FolderSnapshot> = Vec::new();
        let mut failures: IndexMap<LibraryFolderId, StorageError> = IndexMap::new();
        for (index, folder) in folders.iter().enumerate() {
            if progress.is_cancelled() {
                return Err(Stopped::Cancelled);
            }
            let start = ScanProgress {
                folder_name: folder.display_name.clone(),
                folder_number: index + 1,
                folder_count: folders.len(),
                done: 0,
                total: None,
                current: None,
                hashed: false,
            };
            progress.report(&start);
            let adding = add_new_files && folder.role == FolderRole::Primary;
            match self.scan_folder(folder, start, adding, progress)? {
                Ok(snapshot) => snapshots.push(snapshot),
                Err(error) if error.reason == StorageFailure::Cancelled => return Err(Stopped::Cancelled),
                Err(error) => {
                    failures.insert(folder.id.clone(), error);
                }
            }
        }
        let catalog = self.store.current().ok_or(Stopped::LibraryClosed)?.catalog;
        Ok(ScanResult {
            changes: ChangeDetector::detect(&catalog, &snapshots),
            scanned_folders: snapshots.iter().map(|snapshot| snapshot.folder_id.clone()).collect::<IndexSet<_>>(),
            failed_folders: failures,
        })
    }

    /// Scans one folder against the newest catalog; while `adding`, catalogs new files every few seconds.
    fn scan_folder(
        &self,
        folder: &LibraryFolder,
        start: ScanProgress,
        adding: bool,
        progress: &dyn Progress<ScanProgress>,
    ) -> Result<StorageResult<FolderSnapshot>, Stopped> {
        let catalog = self.store.current().ok_or(Stopped::LibraryClosed)?.catalog;
        let listener = FolderListener {
            known_paths: catalog.documents().values().map(|document| document.path.clone()).collect(),
            adding,
            found: Mutex::new(Vec::new()),
            state: Mutex::new(start),
            progress,
        };
        if !adding {
            return Ok(self.scanner.scan(folder, &catalog, &listener));
        }
        let result = thread::scope(|scope| {
            let (stop, stopped) = channel::<()>();
            let found = &listener.found;
            let adder = scope.spawn(move || {
                while let Err(RecvTimeoutError::Timeout) = stopped.recv_timeout(ADD_INTERVAL) {
                    self.add_found(found);
                }
            });
            let result = self.scanner.scan(folder, &catalog, &listener);
            drop(stop);
            // Waits for an add still running, so the last one below never runs at the same time.
            let _ = adder.join();
            result
        });
        self.add_found(&listener.found);
        Ok(result)
    }

    /// Catalogs what was found so far.
    fn add_found(&self, found: &Mutex<Vec<ScannedFile>>) {
        let batch = std::mem::take(&mut *found.lock().unwrap_or_else(PoisonError::into_inner));
        self.add_scanned_files.run(&batch);
    }
}

struct FolderListener<'a> {
    known_paths: HashSet<RelativePath>,
    adding: bool,
    found: Mutex<Vec<ScannedFile>>,
    state: Mutex<ScanProgress>,
    progress: &'a dyn Progress<ScanProgress>,
}

impl ScanListener for FolderListener<'_> {
    fn counted(&self, total: usize) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.total = Some(total);
        self.progress.report(&state);
    }

    fn file_done(&self, done: usize, path: &RelativePath, facts: &FileFacts, hashed: bool) {
        if self.adding
            && let Some(content_hash) = &facts.content_hash
            && !self.known_paths.contains(path)
        {
            let file = ScannedFile { path: path.clone(), size: facts.size, content_hash: content_hash.clone() };
            self.found.lock().unwrap_or_else(PoisonError::into_inner).push(file);
        }
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.done = done;
        state.current = Some(path.clone());
        state.hashed = hashed;
        self.progress.report(&state);
    }

    fn is_cancelled(&self) -> bool {
        self.progress.is_cancelled()
    }
}
