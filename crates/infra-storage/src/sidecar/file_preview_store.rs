use std::fs;
use std::path::Path;

use dms_application::port::sidecar::PreviewStore;
use dms_application::port::storage::StorageResult;
use dms_domain::document::{ContentHash, DocumentId};
use dms_domain::library::LibraryFolder;

use crate::fs::{OrIoError, atomic_files, path_resolver, root_of};
use crate::sidecar::sidecar_paths;
use crate::text::FileTextStore;

/// Hex characters of the content hash in a preview's name.
const HASH_LENGTH: usize = 12;

/// The previews of documents beside their text: `.dms-text/<xx>/<documentId>.<hash12>.jpg`, `hash12` being the
/// first 12 characters of the content hash the preview was rendered for. Writing one removes the previews of other
/// hashes of the document: they are stale files of the app's own, never user files.
#[derive(Debug, Clone, Copy, Default)]
pub struct FilePreviewStore;

impl FilePreviewStore {
    fn suffix(content_hash: &ContentHash) -> String {
        format!(".{}.jpg", &content_hash.value()[..HASH_LENGTH])
    }
}

impl PreviewStore for FilePreviewStore {
    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId, content_hash: &ContentHash) -> Option<Vec<u8>> {
        let root = path_resolver::root(folder)?;
        let file =
            sidecar_paths::subfolder_file(&root, FileTextStore::TEXT_FOLDER, document_id, &Self::suffix(content_hash))?;
        file.is_file().then(|| fs::read(file).ok()).flatten()
    }

    fn write(
        &self,
        folder: &LibraryFolder,
        document_id: &DocumentId,
        content_hash: &ContentHash,
        jpeg: &[u8],
    ) -> StorageResult<()> {
        let root = root_of(folder)?;
        let file =
            sidecar_paths::subfolder_file(&root, FileTextStore::TEXT_FOLDER, document_id, &Self::suffix(content_hash))
                .ok_or_else(|| sidecar_paths::invalid_id(document_id))?;
        atomic_files::write(&file, jpeg).or_io()?;
        remove_stale_previews(&file, document_id);
        Ok(())
    }
}

/// Deletes the other `<documentId>.<hash12>.jpg` files beside `current`. One that cannot be deleted stays: it is
/// never read again, and the next write tries once more.
fn remove_stale_previews(current: &Path, document_id: &DocumentId) {
    let (Some(directory), Some(current_name)) = (current.parent(), current.file_name()) else { return };
    let Ok(entries) = fs::read_dir(directory) else { return };
    let prefix = format!("{}.", document_id.value());
    for entry in entries.flatten() {
        let name = entry.file_name();
        let stale = name != current_name && name.to_str().is_some_and(|name| is_preview_name(name, &prefix));
        if stale {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn is_preview_name(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix).and_then(|rest| rest.strip_suffix(".jpg")).is_some_and(|hash| {
        hash.len() == HASH_LENGTH && hash.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    })
}
