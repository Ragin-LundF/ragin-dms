use dms_domain::document::ContentHash;
use dms_domain::rebuild::RebuiltPage;

/// A rebuilt PDF waiting outside the library for the user's choice. [`Self::id`] names it for the rebuilder only;
/// [`Self::size`] is its size in bytes. [`Self::source_hash`] is the content of the scan it was made from, so a scan
/// that changed since is never replaced by it.
#[derive(Debug, Clone, PartialEq)]
pub struct RebuiltFile {
    pub id: String,
    pub pages: Vec<RebuiltPage>,
    pub size: u64,
    pub source_hash: ContentHash,
}
