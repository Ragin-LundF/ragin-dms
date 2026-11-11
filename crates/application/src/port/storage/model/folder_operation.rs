use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;

/// An operation that brings one sync folder in line with the primary folder. Each one can be recorded in the
/// journal and replayed later, because it only refers to what is in the primary.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FolderOperation {
    /// Copies `path` from the primary. Replaces an existing file only if `replace_existing` is set.
    CopyFromPrimary {
        path: RelativePath,
        replace_existing: bool,
    },
    CreateFolder(RelativePath),
    Move {
        from: RelativePath,
        to: RelativePath,
    },
    MoveToTrash(RelativePath),
    /// Copies the stored text of a document from the primary's `.dms-text/`.
    CopyText(DocumentId),
    /// Copies the embeddings of a document from the primary's `.dms-index/`.
    CopyIndex(DocumentId),
    /// Copies the record of an applied restructure from the primary's `.dms-plans/`.
    CopyPlan(String),
    /// Copies the primary's catalog file as it is at the time of execution.
    SaveCatalog,
}
