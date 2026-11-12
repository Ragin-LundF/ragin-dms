use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Local, NaiveDate};
use dms_application::port::storage::{
    DocumentStorage, ExternalFile, FolderEntries, StorageError, StorageFailure, StorageResult,
};
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};
use indexmap::IndexSet;

use crate::fs::storage_io::{OrIoError, failure};
use crate::fs::{atomic_files, content_hasher, folder_walk, path_resolver};
use crate::platform::SystemTrash;
use crate::scan::CloudPlaceholders;

/// File operations inside one library folder. Never overwrites: an existing entry with the same name, ignoring
/// case and normalization, is a collision, and a replaced file goes to the trash first.
pub struct FileSystemDocumentStorage {
    trash: Arc<dyn SystemTrash>,
}

impl FileSystemDocumentStorage {
    pub fn new(trash: Arc<dyn SystemTrash>) -> Self {
        Self { trash }
    }

    fn trash(&self, root: &Path, entry: &Path) -> StorageResult<()> {
        if self.trash.move_to_trash(root, entry) {
            Ok(())
        } else {
            Err(StorageError::with_detail(StorageFailure::IoError, "Trash failed"))
        }
    }
}

impl DocumentStorage for FileSystemDocumentStorage {
    fn import_file(
        &self,
        folder: &LibraryFolder,
        source: &ExternalFile,
        target: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let root = root_of(folder)?;
        let source = PathBuf::from(source.location());
        if !source.is_file() {
            return Err(failure(StorageFailure::NotFound));
        }
        let target_path = path_resolver::target(&root, target);
        if path_resolver::clashing(parent_of(&target_path), target.file_name()).or_io()?.is_some() {
            return Err(failure(StorageFailure::AlreadyExists));
        }
        atomic_files::copy(&source, &target_path)
    }

    fn copy_file(
        &self,
        from: &LibraryFolder,
        to: &LibraryFolder,
        path: &RelativePath,
        replace_existing: bool,
    ) -> StorageResult<FileFacts> {
        let from_root = root_of(from)?;
        let to_root = root_of(to)?;
        let source = existing_file(&from_root, path)?;
        let target = path_resolver::target(&to_root, path);
        if let Some(existing) = path_resolver::clashing(parent_of(&target), path.file_name()).or_io()? {
            if !replace_existing {
                return Err(failure(StorageFailure::AlreadyExists));
            }
            self.trash(&to_root, &existing)?;
        }
        atomic_files::copy(&source, &target)
    }

    fn create_folder(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<()> {
        let root = root_of(folder)?;
        let target = path_resolver::target(&root, path);
        match path_resolver::clashing(parent_of(&target), path.file_name()).or_io()? {
            None => fs::create_dir_all(&target).or_io(),
            Some(existing) if existing.is_dir() && path_resolver::name_of(&existing) == path.file_name() => Ok(()),
            Some(_) => Err(failure(StorageFailure::AlreadyExists)),
        }
    }

    fn move_entry(&self, folder: &LibraryFolder, from: &RelativePath, to: &RelativePath) -> StorageResult<()> {
        let root = root_of(folder)?;
        let source = path_resolver::existing(&root, from).ok_or_else(|| failure(StorageFailure::NotFound))?;
        let target = path_resolver::target(&root, to);
        let clash = path_resolver::clashing(parent_of(&target), to.file_name()).or_io()?;
        let case_only = clash.as_deref().is_some_and(|clash| same_file(clash, &source));
        if clash.is_some() && !case_only {
            return Err(failure(StorageFailure::AlreadyExists));
        }
        fs::create_dir_all(parent_of(&target)).or_io()?;
        if case_only {
            // A case-insensitive file system treats both names as the same file; go through a third name.
            let intermediate = atomic_files::temp_file_for(&source);
            atomic_files::move_without_replacing(&source, &intermediate)?;
            atomic_files::move_without_replacing(&intermediate, &target)
        } else {
            atomic_files::move_without_replacing(&source, &target)
        }
    }

    fn replace_with(
        &self,
        folder: &LibraryFolder,
        target: &RelativePath,
        replacement: &RelativePath,
    ) -> StorageResult<FileFacts> {
        let root = root_of(folder)?;
        let source = path_resolver::existing(&root, replacement).ok_or_else(|| failure(StorageFailure::NotFound))?;
        let existing = path_resolver::existing(&root, target);
        if let Some(existing) = &existing {
            self.trash(&root, existing)?;
        }
        let destination = existing.unwrap_or_else(|| path_resolver::target(&root, target));
        fs::create_dir_all(parent_of(&destination)).or_io()?;
        // The old file just went to the trash, so the name is free; the move replaces only a file that
        // appeared in this instant, like Kotlin's ATOMIC_MOVE did.
        fs::rename(&source, &destination).or_io()?;
        let size = fs::metadata(&destination).or_io()?.len();
        Ok(FileFacts { size, content_hash: Some(content_hasher::hash(&destination).or_io()?) })
    }

    fn move_to_trash(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<()> {
        let root = root_of(folder)?;
        let entry = path_resolver::existing(&root, path).ok_or_else(|| failure(StorageFailure::NotFound))?;
        self.trash(&root, &entry)
    }

    fn read_facts(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<FileFacts> {
        let root = root_of(folder)?;
        let file = existing_file(&root, path)?;
        let size = fs::metadata(&file).or_io()?.len();
        Ok(FileFacts { size, content_hash: Some(content_hasher::hash(&file).or_io()?) })
    }

    fn read_local_facts(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Option<FileFacts>> {
        let root = root_of(folder)?;
        let Some(file) = path_resolver::existing(&root, path).filter(|file| file.is_file()) else {
            let stub = path_resolver::target(&root, path).with_file_name(format!(".{}.icloud", path.file_name()));
            return if stub.is_file() { Ok(None) } else { Err(failure(StorageFailure::NotFound)) };
        };
        let metadata = fs::metadata(&file).or_io()?;
        if CloudPlaceholders::is_not_downloaded(&metadata) {
            return Ok(None);
        }
        Ok(Some(FileFacts { size: metadata.len(), content_hash: Some(content_hasher::hash(&file).or_io()?) }))
    }

    fn same_content(&self, folder: &LibraryFolder, first: &RelativePath, second: &RelativePath) -> StorageResult<bool> {
        let root = root_of(folder)?;
        let first = existing_file(&root, first)?;
        let second = existing_file(&root, second)?;
        Ok(!same_file(&first, &second) && same_bytes(&first, &second).or_io()?)
    }

    fn created_on(&self, folder: &LibraryFolder, path: &RelativePath) -> Option<NaiveDate> {
        let root = root_of(folder).ok()?;
        let file = existing_file(&root, path).ok()?;
        // File systems without creation times report none, the modification time, or the epoch.
        let created = fs::metadata(file).ok()?.created().ok()?;
        let created: DateTime<Local> = created.into();
        (created.timestamp() != 0 || created.timestamp_subsec_nanos() != 0).then(|| created.date_naive())
    }

    fn list_names(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Vec<String>> {
        let root = root_of(folder)?;
        let directory = existing_directory(&root, path)?;
        fs::read_dir(directory)
            .and_then(|entries| entries.map(|entry| entry.map(|entry| path_resolver::name_of(&entry.path()))).collect())
            .or_io()
    }

    fn list_entries(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<FolderEntries> {
        let root = root_of(folder)?;
        let directory = existing_directory(&root, path)?;
        folder_walk::entries(&root, &directory).or_io()
    }

    fn list_folders(&self, folder: &LibraryFolder) -> StorageResult<IndexSet<RelativePath>> {
        folder_walk::folders(&root_of(folder)?).or_io()
    }
}

/// The folder's root, or `FOLDER_UNREACHABLE`.
pub fn root_of(folder: &LibraryFolder) -> StorageResult<PathBuf> {
    path_resolver::root(folder).ok_or_else(|| failure(StorageFailure::FolderUnreachable))
}

fn existing_file(root: &Path, path: &RelativePath) -> StorageResult<PathBuf> {
    path_resolver::existing(root, path).filter(|file| file.is_file()).ok_or_else(|| failure(StorageFailure::NotFound))
}

fn existing_directory(root: &Path, path: &RelativePath) -> StorageResult<PathBuf> {
    path_resolver::existing(root, path).filter(|file| file.is_dir()).ok_or_else(|| failure(StorageFailure::NotFound))
}

fn parent_of(path: &Path) -> &Path {
    path.parent().unwrap_or(path)
}

/// `Files.isSameFile`: one file under two names, as on a file system that ignores case.
fn same_file(first: &Path, second: &Path) -> bool {
    match (fs::metadata(first), fs::metadata(second)) {
        #[cfg(unix)]
        (Ok(first), Ok(second)) => {
            use std::os::unix::fs::MetadataExt;
            first.dev() == second.dev() && first.ino() == second.ino()
        }
        #[cfg(not(unix))]
        (Ok(_), Ok(_)) => fs::canonicalize(first).ok() == fs::canonicalize(second).ok(),
        _ => false,
    }
}

/// `Files.mismatch(first, second) == -1`: every byte, and so the size, is the same.
fn same_bytes(first: &Path, second: &Path) -> std::io::Result<bool> {
    if fs::metadata(first)?.len() != fs::metadata(second)?.len() {
        return Ok(false);
    }
    let mut first = BufReader::new(File::open(first)?);
    let mut second = BufReader::new(File::open(second)?);
    let (mut left, mut right) = (vec![0; 64 * 1024], vec![0; 64 * 1024]);
    loop {
        let read = first.read(&mut left)?;
        if read == 0 {
            return Ok(true);
        }
        second.read_exact(&mut right[..read])?;
        if left[..read] != right[..read] {
            return Ok(false);
        }
    }
}
