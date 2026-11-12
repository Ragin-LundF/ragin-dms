use chrono::NaiveDate;
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};
use indexmap::IndexSet;

use crate::port::storage::{ExternalFile, FolderEntries, StorageResult};

/// File operations inside one library folder. Implementations never overwrite or delete a file unless the call
/// says so explicitly, write atomically, and create missing parent folders of a target path.
///
/// Every `# Errors` case is a [`crate::port::storage::StorageError`]: the folder is unreachable, a path is missing
/// or taken, or the file system failed.
pub trait DocumentStorage: Send + Sync {
    /// Copies `source` into `folder` at `target` and returns the facts of the written file (hashed).
    ///
    /// # Errors
    /// See the trait documentation.
    fn import_file(
        &self,
        folder: &LibraryFolder,
        source: &ExternalFile,
        target: &RelativePath,
    ) -> StorageResult<FileFacts>;

    /// Copies `path` from one library folder to another, verified by hash. With `replace_existing`, an existing
    /// file at the target is moved to the system trash first, so a replaced version is never lost.
    ///
    /// # Errors
    /// See the trait documentation.
    fn copy_file(
        &self,
        from: &LibraryFolder,
        to: &LibraryFolder,
        path: &RelativePath,
        replace_existing: bool,
    ) -> StorageResult<FileFacts>;

    /// # Errors
    /// See the trait documentation.
    fn create_folder(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<()>;

    /// Moves or renames a file or folder. A rename that only changes the case must work as well.
    ///
    /// # Errors
    /// See the trait documentation.
    fn move_entry(&self, folder: &LibraryFolder, from: &RelativePath, to: &RelativePath) -> StorageResult<()>;

    /// Puts `replacement` in place of `target` in one folder: the old file goes to the system trash, so a replaced
    /// version is never lost, and `replacement` is moved to `target`. Used for a new version of a file, such as an
    /// edited PDF.
    ///
    /// # Errors
    /// See the trait documentation.
    fn replace_with(
        &self,
        folder: &LibraryFolder,
        target: &RelativePath,
        replacement: &RelativePath,
    ) -> StorageResult<FileFacts>;

    /// Moves a file or folder to the system trash.
    ///
    /// # Errors
    /// See the trait documentation.
    fn move_to_trash(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<()>;

    /// Size and content hash of one file; always hashed.
    ///
    /// # Errors
    /// See the trait documentation.
    fn read_facts(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<FileFacts>;

    /// Like [`Self::read_facts`], but `None` for a file a sync client keeps only in the cloud: reading it would
    /// download it, which the archive check must never cause. Storages without such files read every file.
    ///
    /// # Errors
    /// See the trait documentation.
    fn read_local_facts(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Option<FileFacts>> {
        self.read_facts(folder, path).map(Some)
    }

    /// Whether `first` and `second` in `folder` are two files with exactly the same bytes, compared byte by byte.
    /// Two names of one file, as on a file system that ignores case, are not two files.
    ///
    /// # Errors
    /// See the trait documentation.
    fn same_content(&self, folder: &LibraryFolder, first: &RelativePath, second: &RelativePath) -> StorageResult<bool>;

    /// The day the file was created, as far as the file system knows it, or `None` when it does not keep creation
    /// times or the file is missing.
    fn created_on(&self, folder: &LibraryFolder, path: &RelativePath) -> Option<NaiveDate>;

    /// Names of the entries directly inside `path`, for collision checks.
    ///
    /// # Errors
    /// See the trait documentation.
    fn list_names(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Vec<String>>;

    /// Every folder and file below `path`, at any depth, as paths from the root; the tool's own files and system
    /// clutter are left out, as in a scan.
    ///
    /// # Errors
    /// `NotFound` when `path` is no folder; otherwise see the trait documentation.
    fn list_entries(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<FolderEntries>;

    /// Every folder below the top level of `folder`, at any depth, also empty ones; files are not read. Left out
    /// as in a scan: the tool's own folders and system clutter.
    ///
    /// # Errors
    /// See the trait documentation.
    fn list_folders(&self, folder: &LibraryFolder) -> StorageResult<IndexSet<RelativePath>>;
}
