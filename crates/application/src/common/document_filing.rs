use std::sync::Arc;

use dms_domain::document::DocumentId;
use dms_domain::library::{FileNamePolicy, RelativePath};

use crate::common::{OperationOutcome, SiblingNames};
use crate::document::MoveDocument;
use crate::library::CatalogStore;

/// Files a document into a folder of the library, in every library folder: missing folders are created on the
/// way, and a name another entry of the target folder already has gets a ` (2)` suffix, so filing never fails on
/// a clash. Renaming within the same folder keeps the strict rule of [`MoveDocument`] instead: a taken name is
/// refused.
pub struct DocumentFiling {
    store: Arc<CatalogStore>,
    names: Arc<SiblingNames>,
    move_document: Arc<MoveDocument>,
}

impl DocumentFiling {
    pub fn new(store: Arc<CatalogStore>, names: Arc<SiblingNames>, move_document: Arc<MoveDocument>) -> Self {
        Self { store, names, move_document }
    }

    pub fn file(&self, document_id: &DocumentId, folder: &RelativePath, file_name: Option<&str>) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return OperationOutcome::NotFound;
        };
        let name = file_name.unwrap_or_else(|| document.path.file_name());
        let violation = folder.segments().into_iter().chain([name]).find_map(FileNamePolicy::check);
        if let Some(violation) = violation {
            return OperationOutcome::InvalidName(violation);
        }
        let current_folder = document.path.parent().unwrap_or_else(RelativePath::root);
        if *folder == current_folder {
            return self.move_to(document_id, folder, name);
        }
        let siblings = match self.names.of(&library, folder) {
            Ok(siblings) => siblings,
            Err(error) => return OperationOutcome::StorageFailed(error),
        };
        let siblings: Vec<&str> = siblings.iter().map(String::as_str).collect();
        let free = FileNamePolicy::next_free_name(name, &siblings);
        self.move_to(document_id, folder, &free)
    }

    fn move_to(&self, document_id: &DocumentId, folder: &RelativePath, name: &str) -> OperationOutcome {
        match folder.resolve(name) {
            Ok(path) => self.move_document.run(document_id, &path),
            Err(error) => OperationOutcome::InvalidInput(error),
        }
    }
}
