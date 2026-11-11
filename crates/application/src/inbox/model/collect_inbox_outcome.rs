use dms_domain::change::NewFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectInboxOutcome {
    /// The changes that became inbox documents, so the caller can drop them from the scan; none while the inbox is
    /// off.
    Collected(Vec<NewFile>),
    LibraryNotOpen,
}
