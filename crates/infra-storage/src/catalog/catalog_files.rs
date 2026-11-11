use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dms_domain::text;

use crate::fs::path_resolver;

/// The catalog file and the conflict copies sync clients create next to it, such as `dms-catalog 2.yaml`,
/// `dms-catalog (conflicted copy).yaml` or `dms-catalog.sync-conflict-….yaml`.
pub struct CatalogFiles;

const PREFIX: &str = "dms-catalog";
const EXTENSION: &str = ".yaml";

impl CatalogFiles {
    pub const FILE_NAME: &'static str = "dms-catalog.yaml";

    /// `true` for the catalog file and all of its conflict copies.
    #[must_use]
    pub fn is_catalog_file(name: &str) -> bool {
        let lower = text::lowercase(name);
        lower.starts_with(PREFIX) && lower.ends_with(EXTENSION)
    }

    /// The conflict copies in `root`, sorted by name.
    ///
    /// # Errors
    /// Fails when the folder cannot be listed.
    pub fn conflict_copies(root: &Path) -> io::Result<Vec<PathBuf>> {
        let mut copies = Vec::new();
        for entry in fs::read_dir(root)? {
            let path = entry?.path();
            let name = path_resolver::name_of(&path);
            if path.is_file() && Self::is_catalog_file(&name) && name != Self::FILE_NAME {
                copies.push(path);
            }
        }
        copies.sort_by(|first, second| text::compare(&raw_name(first), &raw_name(second)));
        Ok(copies)
    }
}

fn raw_name(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}
