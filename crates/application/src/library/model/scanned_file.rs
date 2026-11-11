use dms_domain::document::ContentHash;
use dms_domain::library::RelativePath;

/// A file the scan of the primary folder read and the catalog did not know at the start of the scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedFile {
    pub path: RelativePath,
    pub size: u64,
    pub content_hash: ContentHash,
}
