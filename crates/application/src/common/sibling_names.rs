use std::sync::Arc;

use dms_domain::library::{FileNamePolicy, RelativePath};
use indexmap::IndexSet;

use crate::common::LibrarySnapshot;
use crate::port::storage::{DocumentStorage, StorageFailure, StorageResult};

/// The names a new entry in a folder must not clash with: what the primary folder holds there, plus what the
/// catalog places there, because a sync folder may still hold documents the primary lacks.
pub struct SiblingNames {
    storage: Arc<dyn DocumentStorage>,
}

impl SiblingNames {
    pub fn new(storage: Arc<dyn DocumentStorage>) -> Self {
        Self { storage }
    }

    /// The names directly inside `parent`, or the failure that kept the primary from listing them.
    ///
    /// # Errors
    /// The primary could not be listed for another reason than a missing folder.
    pub fn of(&self, library: &LibrarySnapshot, parent: &RelativePath) -> StorageResult<IndexSet<String>> {
        let on_disk = match self.storage.list_names(library.folders.primary(), parent) {
            Ok(names) => names,
            Err(error) if error.reason == StorageFailure::NotFound => Vec::new(),
            Err(error) => return Err(error),
        };
        let depth = parent.segments().len();
        let catalogued = library
            .catalog
            .documents_inside(parent)
            .into_iter()
            .map(|document| document.path.segments()[depth].to_owned());
        Ok(on_disk.into_iter().chain(catalogued).collect())
    }

    /// `true` if nothing else in its folder has the name of `target`. `moving` is the entry being moved or
    /// renamed there, which does not clash with itself, so a rename that only changes the case works.
    ///
    /// # Errors
    /// See [`Self::of`].
    pub fn is_free(
        &self,
        library: &LibrarySnapshot,
        target: &RelativePath,
        moving: Option<&RelativePath>,
    ) -> StorageResult<bool> {
        let parent = target.parent().unwrap_or_else(RelativePath::root);
        let siblings = self.of(library, &parent)?;
        let others = siblings.iter().filter(|name| {
            !moving.is_some_and(|moving| {
                moving.parent().as_ref() == Some(&parent) && FileNamePolicy::is_same_name(name, moving.file_name())
            })
        });
        Ok(!FileNamePolicy::collides_with_any(target.file_name(), others))
    }
}
