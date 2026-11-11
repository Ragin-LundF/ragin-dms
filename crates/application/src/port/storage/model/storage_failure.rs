/// Why a file system operation in a library folder failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageFailure {
    /// The folder is not reachable right now, for example an offline network share.
    FolderUnreachable,
    AlreadyExists,
    NotFound,
    /// The copy was written but its content hash does not match the source.
    VerificationFailed,
    IoError,
    /// A file written by a newer version of the tool, or not a file this tool can read.
    UnsupportedFormat,
    /// The catalog file holds a newer revision than the change was based on; nothing was written.
    CatalogChanged,
    /// The caller asked to stop, for example a scan through [`crate::port::storage::ScanListener::is_cancelled`].
    /// Kotlin had no value for this: a cancelled coroutine simply stopped.
    Cancelled,
}
