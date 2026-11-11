use crate::sidecar::FolderMigration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarMigrationOutcome {
    LibraryNotOpen,
    /// Every library folder, in order.
    Done(Vec<FolderMigration>),
    /// Stopped before the next file; the folders looked at so far, the last one unfinished.
    Cancelled(Vec<FolderMigration>),
}
