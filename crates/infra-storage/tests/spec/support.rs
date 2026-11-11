//! Library folders and files on a temporary directory, with synthetic names only.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use dms_application::port::storage::{ScanListener, StorageResult};
use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogUpdate};
use dms_domain::change::FileFacts;
use dms_domain::document::ContentHash;
use dms_domain::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId, RelativePath};
use dms_infra_storage::SystemTrash;
use sha2::{Digest, Sha256};

pub fn folder(location: &Path) -> LibraryFolder {
    folder_with(location, "primary", FolderRole::Primary)
}

pub fn folder_with(location: &Path, id: &str, role: FolderRole) -> LibraryFolder {
    LibraryFolder::new(
        LibraryFolderId::new(id).expect("id"),
        id,
        FolderLocation::new(location.to_string_lossy()).expect("location"),
        role,
    )
    .expect("folder")
}

pub fn write(file: &Path, content: &str) -> PathBuf {
    fs::create_dir_all(file.parent().expect("parent")).expect("parent folder");
    fs::write(file, content).expect("file");
    file.to_path_buf()
}

pub fn read(file: &Path) -> String {
    fs::read_to_string(file).expect("readable file")
}

pub fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("valid path")
}

pub fn hash_of(content: &str) -> ContentHash {
    let hex = Sha256::digest(content.as_bytes()).iter().fold(String::new(), |hex, byte| hex + &format!("{byte:02x}"));
    ContentHash::new(hex).expect("hash")
}

/// Names of the entries directly inside `directory`, as stored on disk.
pub fn names_in(directory: &Path) -> BTreeSet<String> {
    fs::read_dir(directory)
        .expect("directory")
        .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
        .collect()
}

pub fn value_of<T: std::fmt::Debug>(result: StorageResult<T>) -> T {
    result.unwrap_or_else(|error| panic!("expected success, got {error:?}"))
}

pub fn failure_of<T>(result: StorageResult<T>) -> Option<dms_application::port::storage::StorageFailure> {
    result.err().map(|error| error.reason)
}

pub fn fixture(name: &str) -> String {
    read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name))
}

/// A trash in a test directory that remembers what it received; with `accepts` false it refuses everything.
pub struct RecordingTrash {
    directory: PathBuf,
    accepts: bool,
    received: Mutex<Vec<(String, PathBuf)>>,
}

impl RecordingTrash {
    pub fn new(directory: &Path) -> Self {
        Self { directory: directory.to_path_buf(), accepts: true, received: Mutex::new(Vec::new()) }
    }

    pub fn refusing(directory: &Path) -> Self {
        Self { accepts: false, ..Self::new(directory) }
    }

    /// Names of the trashed entries, in the order they arrived.
    pub fn trashed(&self) -> Vec<String> {
        self.received.lock().expect("lock").iter().map(|(name, _)| name.clone()).collect()
    }

    /// The content of the trashed file that had `name`.
    pub fn content_of(&self, name: &str) -> Option<String> {
        let received = self.received.lock().expect("lock");
        received.iter().find(|(trashed, _)| trashed == name).map(|(_, target)| read(target))
    }
}

impl SystemTrash for RecordingTrash {
    fn move_to_trash(&self, _library_root: &Path, entry: &Path) -> bool {
        if !self.accepts {
            return false;
        }
        fs::create_dir_all(&self.directory).expect("trash folder");
        let mut received = self.received.lock().expect("lock");
        let name = entry.file_name().expect("name").to_string_lossy().into_owned();
        let target = self.directory.join(format!("{}-{name}", received.len()));
        fs::rename(entry, &target).expect("move to trash");
        received.push((name, target));
        true
    }
}

/// Records what a scan reports, as `"<done> <path> <hashed>"` per file; stops after `cancel_after` files.
#[derive(Default)]
pub struct RecordingListener {
    pub totals: Mutex<Vec<usize>>,
    pub files: Mutex<Vec<String>>,
    pub cancel_after: Option<usize>,
}

impl RecordingListener {
    pub fn files(&self) -> Vec<String> {
        self.files.lock().expect("lock").clone()
    }
}

impl ScanListener for RecordingListener {
    fn counted(&self, total: usize) {
        self.totals.lock().expect("lock").push(total);
    }

    fn file_done(&self, done: usize, path: &RelativePath, _facts: &FileFacts, hashed: bool) {
        self.files.lock().expect("lock").push(format!("{done} {} {hashed}", path.value()));
    }

    fn is_cancelled(&self) -> bool {
        self.cancel_after.is_some_and(|after| self.files.lock().expect("lock").len() >= after)
    }
}

/// Unwraps an applied update and fails the test for a rejection.
pub trait Applied {
    fn applied(self) -> Catalog;
}

impl Applied for CatalogUpdate {
    fn applied(self) -> Catalog {
        match self {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("expected an applied update, got {reason:?}"),
        }
    }
}

impl Applied for Result<CatalogUpdate, DomainError> {
    fn applied(self) -> Catalog {
        self.expect("valid change").applied()
    }
}
