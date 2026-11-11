use std::sync::{Arc, Mutex, PoisonError};

use dms_domain::library::{LibraryFolderId, RelativePath};
use indexmap::IndexMap;

use crate::library::CatalogStore;
use crate::port::ai::{DocumentPreviewer, PageImage};
use crate::preview::ThumbnailOutcome;

const CAPACITY: usize = 200;
/// Sharp enough for a small picture on a high-resolution screen.
const MAX_SIDE: u32 = 320;

type Key = (LibraryFolderId, RelativePath);

/// Thumbnails of the files in the library folders, also of files the catalog does not know yet. The most recent ones
/// are kept in memory, so a list drawn again does not render its files again; a file without a picture is
/// remembered as such. Renders one file at a time and never writes.
pub struct DocumentThumbnails {
    store: Arc<CatalogStore>,
    previewer: Arc<dyn DocumentPreviewer>,
    /// Least recently drawn first, like Kotlin's access-ordered `LinkedHashMap`.
    cache: Mutex<IndexMap<Key, Option<PageImage>>>,
}

impl DocumentThumbnails {
    pub fn new(store: Arc<CatalogStore>, previewer: Arc<dyn DocumentPreviewer>) -> Self {
        Self { store, previewer, cache: Mutex::new(IndexMap::new()) }
    }

    pub fn run(&self, folder_id: &LibraryFolderId, path: &RelativePath) -> ThumbnailOutcome {
        let Some(folder) = self.store.current().and_then(|library| library.folders.find(folder_id).cloned()) else {
            return ThumbnailOutcome::FolderNotFound;
        };
        let key = (folder_id.clone(), path.clone());
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let image = if let Some(image) = cache.shift_remove(&key) {
            cache.insert(key, image.clone());
            image
        } else {
            let image = self.previewer.thumbnail(&folder, path, MAX_SIDE);
            cache.insert(key, image.clone());
            if cache.len() > CAPACITY {
                cache.shift_remove_index(0);
            }
            image
        };
        image.map_or(ThumbnailOutcome::NoPicture, ThumbnailOutcome::Rendered)
    }
}
