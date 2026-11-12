use chrono::{DateTime, Utc};
use dms_domain::catalog::LibraryId;
use dms_domain::document::DocumentId;
use dms_domain::health::HealthFinding;
use dms_domain::library::LibraryFolderId;

/// What the archive check of this device knows: when it ran, how far a pass got, per folder what it read, and the
/// findings that wait for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    pub library_id: LibraryId,
    /// When the current or last pass started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the last complete pass ended; schedules count from here.
    pub finished_at: Option<DateTime<Utc>>,
    /// Set while a pass is unfinished: it continues after this document (ids in ascending order).
    pub resume_after: Option<DocumentId>,
    pub folders: Vec<FolderCheck>,
    pub findings: Vec<HealthFinding>,
}

impl HealthReport {
    #[must_use]
    pub fn new(library_id: LibraryId) -> Self {
        Self {
            library_id,
            started_at: None,
            finished_at: None,
            resume_after: None,
            folders: Vec::new(),
            findings: Vec::new(),
        }
    }
}

/// What the current or last pass did in one folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderCheck {
    pub folder_id: LibraryFolderId,
    /// Files read and compared with the catalog.
    pub checked: u32,
    /// Files a sync client keeps only in the cloud; skipped, never downloaded.
    pub not_downloaded: u32,
    pub missing: u32,
    /// The folder was offline; its last findings stay and it is checked next time.
    pub offline: bool,
}

impl FolderCheck {
    #[must_use]
    pub fn new(folder_id: LibraryFolderId) -> Self {
        Self { folder_id, checked: 0, not_downloaded: 0, missing: 0, offline: false }
    }
}
