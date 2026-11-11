use dms_domain::document::{DocumentId, DocumentMetadata};
use dms_domain::taxonomy::{CategoryId, TagName};
use indexmap::IndexSet;

/// A change to the categorization in the catalog that touches no file.
#[derive(Debug, Clone, PartialEq)]
pub enum CatalogEdit {
    UpdateMetadata {
        document_id: DocumentId,
        metadata: DocumentMetadata,
    },
    /// Sets the category of a document, or clears it with `None`.
    AssignCategory {
        document_id: DocumentId,
        category_id: Option<CategoryId>,
    },
    AddTags {
        document_id: DocumentId,
        tags: IndexSet<TagName>,
    },
    RemoveTags {
        document_id: DocumentId,
        tags: IndexSet<TagName>,
    },
    /// Creates a category; its id is generated.
    AddCategory {
        name: String,
        parent_id: Option<CategoryId>,
    },
    RenameCategory {
        category_id: CategoryId,
        name: String,
    },
    MoveCategory {
        category_id: CategoryId,
        parent_id: Option<CategoryId>,
    },
    RemoveCategory {
        category_id: CategoryId,
    },
    /// Renames a tag everywhere; renaming to an existing tag merges the two.
    RenameTag {
        from: TagName,
        to: TagName,
    },
    RemoveTag {
        tag: TagName,
    },
}

impl CatalogEdit {
    /// The name of the edit, as Kotlin's `edit::class.simpleName` wrote it into the activity log.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::UpdateMetadata { .. } => "UpdateMetadata",
            Self::AssignCategory { .. } => "AssignCategory",
            Self::AddTags { .. } => "AddTags",
            Self::RemoveTags { .. } => "RemoveTags",
            Self::AddCategory { .. } => "AddCategory",
            Self::RenameCategory { .. } => "RenameCategory",
            Self::MoveCategory { .. } => "MoveCategory",
            Self::RemoveCategory { .. } => "RemoveCategory",
            Self::RenameTag { .. } => "RenameTag",
            Self::RemoveTag { .. } => "RemoveTag",
        }
    }
}
