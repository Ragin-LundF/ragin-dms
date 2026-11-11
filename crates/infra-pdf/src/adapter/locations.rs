use std::fs;
use std::path::{Path, PathBuf};

use dms_domain::library::{LibraryFolder, RelativePath};
use unicode_normalization::UnicodeNormalization;

/// Where library files are on this device, as Kotlin's `PathResolver`: [`existing`](Self::existing) finds a file
/// that is there, [`target`](Self::target) names the place of a new one; both `None` while the folder is not
/// reachable. macOS may report names decomposed (NFD) while the catalog stores them composed (NFC), so existing
/// entries are also found by their NFC name.
#[derive(Debug, Clone, Copy, Default)]
pub struct LibraryLocations;

impl LibraryLocations {
    /// The existing entry for `path` in `folder`, or `None` if there is none or the folder is not reachable.
    #[must_use]
    pub fn existing(self, folder: &LibraryFolder, path: &RelativePath) -> Option<PathBuf> {
        let mut current = root_of(folder)?;
        for segment in path.segments() {
            current = child(&current, segment)?;
        }
        Some(current)
    }

    /// Where `path` lives or would be created in `folder`: existing folders are matched by NFC name, the remaining
    /// segments are used as given. `None` if the folder is not reachable.
    #[must_use]
    pub fn target(self, folder: &LibraryFolder, path: &RelativePath) -> Option<PathBuf> {
        let mut current = root_of(folder)?;
        let mut parent_exists = true;
        for segment in path.segments() {
            let found = if parent_exists { child(&current, segment) } else { None };
            parent_exists = found.is_some();
            current = found.unwrap_or_else(|| current.join(segment));
        }
        Some(current)
    }
}

fn root_of(folder: &LibraryFolder) -> Option<PathBuf> {
    let root = PathBuf::from(folder.location.value());
    root.is_dir().then_some(root)
}

/// The entry of `directory` named `name` (NFC), as it is spelled on disk.
fn child(directory: &Path, name: &str) -> Option<PathBuf> {
    let direct = directory.join(name);
    if fs::symlink_metadata(&direct).is_ok() {
        return Some(direct);
    }
    fs::read_dir(directory)
        .ok()?
        .flatten()
        .find(|entry| entry.file_name().to_string_lossy().nfc().eq(name.chars()))
        .map(|entry| entry.path())
}
