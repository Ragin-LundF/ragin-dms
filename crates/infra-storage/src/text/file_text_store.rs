use std::fs;
use std::path::{Path, PathBuf};

use dms_application::port::storage::{StorageResult, TextStore};
use dms_domain::document::DocumentId;
use dms_domain::library::LibraryFolder;

use crate::fs::{OrIoError, atomic_files, path_resolver, root_of};

/// The text of documents in `.dms-text/<documentId>.txt`, UTF-8. Named by id, so renaming or moving a document
/// never touches it. Derived data: a missing file is not an error.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileTextStore;

impl FileTextStore {
    pub const TEXT_FOLDER: &'static str = ".dms-text";

    fn file_of(root: &Path, document_id: &DocumentId) -> PathBuf {
        root.join(Self::TEXT_FOLDER).join(format!("{}.txt", document_id.value()))
    }
}

impl TextStore for FileTextStore {
    fn write(&self, folder: &LibraryFolder, document_id: &DocumentId, text: &str) -> StorageResult<()> {
        let root = root_of(folder)?;
        atomic_files::write(&Self::file_of(&root, document_id), text.as_bytes()).or_io()
    }

    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<String> {
        let file = Self::file_of(&path_resolver::root(folder)?, document_id);
        // Like Java's `Files.readString`, malformed UTF-8 is an error, so it reads as no text.
        file.is_file().then(|| fs::read_to_string(file).ok()).flatten()
    }
}
