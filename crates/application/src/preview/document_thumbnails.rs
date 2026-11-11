use std::sync::{Arc, Mutex, PoisonError};

use dms_domain::document::{ContentHash, Document};
use dms_domain::library::{LibraryFolder, LibraryFolderId, LibraryFolders, RelativePath};
use indexmap::IndexMap;

use crate::library::CatalogStore;
use crate::port::ai::{DocumentPreviewer, PageImage};
use crate::port::sidecar::PreviewStore;
use crate::preview::ThumbnailOutcome;

const CAPACITY: usize = 200;
/// Sharp enough for a small picture on a high-resolution screen.
const MAX_SIDE: u32 = 320;
const JPEG: &str = "image/jpeg";
/// A preview is stored only while it is smaller than a tenth of its document, so small files do not make the
/// library grow by more than their previews are worth.
const SIZE_DIVISOR: u64 = 10;

/// The catalogued document's content hash is part of the key, so a changed document is drawn again.
type Key = (LibraryFolderId, RelativePath, Option<ContentHash>);

/// Thumbnails of the files in the library folders, also of files the catalog does not know yet. The most recent ones
/// are kept in memory, so a list drawn again does not render its files again; a file without a picture is
/// remembered as such. Renders one file at a time.
///
/// A catalogued document's thumbnail is also stored in every library folder reachable right now (see
/// [`PreviewStore`]) for its current content hash, so the next start does not render it again; one that would take a
/// tenth of the document's size or more stays in memory only. Previews are cheap to make again, so a folder that
/// cannot store one is not journaled: it renders its own preview when it is drawn from there.
pub struct DocumentThumbnails {
    store: Arc<CatalogStore>,
    previewer: Arc<dyn DocumentPreviewer>,
    previews: Arc<dyn PreviewStore>,
    /// Least recently drawn first, like Kotlin's access-ordered `LinkedHashMap`.
    cache: Mutex<IndexMap<Key, Option<PageImage>>>,
}

impl DocumentThumbnails {
    pub fn new(
        store: Arc<CatalogStore>,
        previewer: Arc<dyn DocumentPreviewer>,
        previews: Arc<dyn PreviewStore>,
    ) -> Self {
        Self { store, previewer, previews, cache: Mutex::new(IndexMap::new()) }
    }

    pub fn run(&self, folder_id: &LibraryFolderId, path: &RelativePath) -> ThumbnailOutcome {
        let Some(library) = self.store.current() else {
            return ThumbnailOutcome::FolderNotFound;
        };
        let Some(folder) = library.folders.find(folder_id) else {
            return ThumbnailOutcome::FolderNotFound;
        };
        let document = library.catalog.find_by_path(path);
        let key = (folder_id.clone(), path.clone(), document.map(|document| document.content_hash.clone()));
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let image = if let Some(image) = cache.shift_remove(&key) {
            cache.insert(key, image.clone());
            image
        } else {
            let image = match document {
                Some(document) => self.stored_or_rendered(&library.folders, folder, document),
                None => self.previewer.thumbnail(folder, path, MAX_SIDE),
            };
            cache.insert(key, image.clone());
            if cache.len() > CAPACITY {
                cache.shift_remove_index(0);
            }
            image
        };
        image.map_or(ThumbnailOutcome::NoPicture, ThumbnailOutcome::Rendered)
    }

    fn stored_or_rendered(
        &self,
        folders: &LibraryFolders,
        folder: &LibraryFolder,
        document: &Document,
    ) -> Option<PageImage> {
        if let Some(jpeg) = self.previews.read(folder, &document.id, &document.content_hash) {
            return Some(PageImage { bytes: jpeg, mime_type: JPEG.to_owned() });
        }
        let image = self.previewer.thumbnail(folder, &document.path, MAX_SIDE)?;
        let small =
            u64::try_from(image.bytes.len()).is_ok_and(|size| size.saturating_mul(SIZE_DIVISOR) < document.size);
        if image.mime_type == JPEG && small {
            for target in folders.folders() {
                // An offline folder simply renders its own preview later.
                let _ = self.previews.write(target, &document.id, &document.content_hash, &image.bytes);
            }
        }
        Some(image)
    }
}
