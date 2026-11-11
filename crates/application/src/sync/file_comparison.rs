use dms_domain::catalog::Catalog;
use dms_domain::change::FileFacts;
use dms_domain::document::ContentHash;
use dms_domain::library::RelativePath;

/// Compares the copies of one path in different folders, as far as a scan can tell.
pub struct FileComparison;

impl FileComparison {
    /// The content hash of a file. A scanner hashes only files that changed since the last scan, so a missing hash
    /// of a catalogued file means it still has the catalogued content.
    #[must_use]
    pub fn effective_hash<'a>(
        facts: &'a FileFacts,
        path: &RelativePath,
        catalog: &'a Catalog,
    ) -> Option<&'a ContentHash> {
        facts.content_hash.as_ref().or_else(|| catalog.find_by_path(path).map(|document| &document.content_hash))
    }

    #[must_use]
    pub fn differs(first: &FileFacts, second: &FileFacts, path: &RelativePath, catalog: &Catalog) -> bool {
        let first_hash = Self::effective_hash(first, path, catalog);
        let second_hash = Self::effective_hash(second, path, catalog);
        if let (Some(first_hash), Some(second_hash)) = (first_hash, second_hash) {
            return first_hash != second_hash;
        }
        first.size != second.size
    }
}
