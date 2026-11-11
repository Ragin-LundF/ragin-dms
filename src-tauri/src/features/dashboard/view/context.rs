//! Names of folders, documents and categories for the texts and buttons of tasks (Kotlin `TaskContext`).

use dms_domain::catalog::Catalog;
use dms_domain::change::NewFile;
use dms_domain::document::{Document, DocumentId};
use dms_domain::field::FieldKey;
use dms_domain::library::{FolderRole, LibraryFolder, LibraryFolderId, RelativePath};
use dms_domain::taxonomy::CategoryId;

pub struct TaskContext<'a> {
    folders: Vec<&'a LibraryFolder>,
    catalog: &'a Catalog,
}

impl<'a> TaskContext<'a> {
    pub fn new(folders: Vec<&'a LibraryFolder>, catalog: &'a Catalog) -> Self {
        Self { folders, catalog }
    }

    pub fn catalog(&self) -> &'a Catalog {
        self.catalog
    }

    pub fn folders(&self) -> &[&'a LibraryFolder] {
        &self.folders
    }

    pub fn primary_id(&self) -> Option<&'a LibraryFolderId> {
        self.folders.iter().find(|folder| folder.role == FolderRole::Primary).map(|folder| &folder.id)
    }

    pub fn folder_name(&self, id: &LibraryFolderId) -> String {
        self.folders.iter().find(|folder| &folder.id == id).map_or_else(|| id.to_string(), |f| f.display_name.clone())
    }

    /// The names joined by ", ", or "no reachable folder" for none.
    pub fn folder_names<'i>(&self, ids: impl IntoIterator<Item = &'i LibraryFolderId>) -> String {
        let names: Vec<String> = ids.into_iter().map(|id| self.folder_name(id)).collect();
        if names.is_empty() { "no reachable folder".to_owned() } else { names.join(", ") }
    }

    pub fn document_path(&self, id: &DocumentId) -> String {
        self.path_of(id).map_or_else(|| id.to_string(), ToString::to_string)
    }

    pub fn path_of(&self, id: &DocumentId) -> Option<&'a RelativePath> {
        self.catalog.documents().get(id).map(|document| &document.path)
    }

    /// `ids` with the primary folder first, otherwise in their order.
    pub fn primary_first<'i>(&self, ids: impl IntoIterator<Item = &'i LibraryFolderId>) -> Vec<&'i LibraryFolderId> {
        let primary = self.primary_id();
        let mut sorted: Vec<&LibraryFolderId> = ids.into_iter().collect();
        sorted.sort_by_key(|id| Some(*id) != primary);
        sorted
    }

    /// The catalogued document `change` copies, by size and SHA-256. A file at a path the catalog knows is that
    /// document itself, not a copy.
    pub fn original_of(&self, change: &NewFile) -> Option<&'a Document> {
        let hash = change.content_hash.as_ref()?;
        if self.catalog.find_by_path(&change.path).is_some() {
            return None;
        }
        self.catalog.find_by_content_hash(hash).into_iter().find(|document| document.size == change.size)
    }

    /// The documents of `ids` the catalog still knows.
    pub fn documents(&self, ids: &[DocumentId]) -> Vec<&'a Document> {
        ids.iter().filter_map(|id| self.catalog.documents().get(id)).collect()
    }

    pub fn field_label(&self, key: &FieldKey) -> String {
        self.catalog.settings().field(key).map_or_else(|| key.to_string(), |field| field.label.clone())
    }

    pub fn category_name(&self, id: &CategoryId) -> String {
        self.catalog.categories().get(id).map_or_else(|| id.to_string(), |category| category.name.clone())
    }

    /// A folder other than `folder_id`, the primary first.
    pub fn other_folder_than(&self, folder_id: &LibraryFolderId) -> Option<&'a LibraryFolderId> {
        let ids: Vec<&'a LibraryFolderId> = self.folders.iter().map(|folder| &folder.id).collect();
        self.primary_first(ids).into_iter().find(|id| *id != folder_id)
    }
}
