/// What the tool did, for the dashboard's recent activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityKind {
    DocumentAdded,
    DocumentMoved,
    DocumentFiled,
    DocumentTrashed,
    DocumentRestored,
    FolderCreated,
    FolderMoved,
    FolderTrashed,
    CatalogEdited,
    SuggestionApproved,
    ConflictResolved,
    Synchronized,
    Restructured,
    Exported,
    PdfEdited,
}
