use crate::document::ContentHash;

/// What a scan found out about one file.
///
/// [`Self::content_hash`] is `None` when the scanner skipped hashing because the file is unchanged since its last
/// scan, so that a scan does not download files a sync client keeps only remotely. Files the catalog does not
/// know are always hashed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileFacts {
    pub size: u64,
    pub content_hash: Option<ContentHash>,
}
