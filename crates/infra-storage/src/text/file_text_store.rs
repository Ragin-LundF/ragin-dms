use std::fs;

use dms_application::port::storage::{StorageResult, TextStore};
use dms_domain::document::DocumentId;
use dms_domain::library::LibraryFolder;

use crate::fs::{OrIoError, atomic_files, path_resolver, root_of};
use crate::sidecar::sidecar_paths;

/// The text of documents in `.dms-text/<xx>/<documentId>.txt` (see [`sidecar_paths`]), UTF-8; a folder not moved to
/// subfolders yet is read from `.dms-text/<documentId>.txt`. Named by id, so renaming or moving a document never
/// touches it. Derived data: a missing file is not an error.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileTextStore;

impl FileTextStore {
    pub const TEXT_FOLDER: &'static str = ".dms-text";
    const SUFFIX: &'static str = ".txt";
}

impl TextStore for FileTextStore {
    fn write(&self, folder: &LibraryFolder, document_id: &DocumentId, text: &str) -> StorageResult<()> {
        let root = root_of(folder)?;
        let file = sidecar_paths::subfolder_file(&root, Self::TEXT_FOLDER, document_id, Self::SUFFIX)
            .ok_or_else(|| sidecar_paths::invalid_id(document_id))?;
        atomic_files::write(&file, text.as_bytes()).or_io()
    }

    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<String> {
        let file =
            sidecar_paths::existing_file(&path_resolver::root(folder)?, Self::TEXT_FOLDER, document_id, Self::SUFFIX)?;
        // Like Java's `Files.readString`, malformed UTF-8 is an error, so it reads as no text.
        fs::read_to_string(file).ok()
    }
}
