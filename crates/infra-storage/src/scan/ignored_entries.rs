use dms_domain::library::TemporaryNames;

use crate::catalog::CatalogFiles;
use crate::index::FileVectorIndex;
use crate::plan::FilePlanStore;
use crate::platform::PlatformTrash;
use crate::text::FileTextStore;

/// Files and folders a scan skips: system clutter, the tool's own files and unfinished writes.
pub struct IgnoredEntries;

const NAMES: [&str; 11] = [
    ".DS_Store",
    ".Spotlight-V100",
    ".Trashes",
    ".fseventsd",
    "Icon\r",
    "Thumbs.db",
    "desktop.ini",
    FileTextStore::TEXT_FOLDER,
    FilePlanStore::PLANS_FOLDER,
    FileVectorIndex::INDEX_FOLDER,
    PlatformTrash::FALLBACK_FOLDER,
];

impl IgnoredEntries {
    #[must_use]
    pub fn is_ignored(name: &str, at_root: bool) -> bool {
        NAMES.contains(&name)
            || name.starts_with("._")
            || TemporaryNames::is_temporary(name)
            || (at_root && CatalogFiles::is_catalog_file(name))
    }
}
