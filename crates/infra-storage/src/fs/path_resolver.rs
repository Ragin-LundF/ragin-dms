//! The one mapping between [`RelativePath`] (NFC, `/`) and file system paths.
//!
//! macOS may report names decomposed (NFD) while the catalog stores them composed (NFC), so existing entries are
//! found by comparing NFC-normalized names, never by string equality alone.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dms_domain::library::{FileNamePolicy, LibraryFolder, RelativePath};
use dms_domain::text;

/// The folder's root directory, or `None` if it is not reachable right now.
pub fn root(folder: &LibraryFolder) -> Option<PathBuf> {
    let root = PathBuf::from(folder.location.value());
    root.is_dir().then_some(root)
}

/// The existing entry for `path` below `root`, or `None` if there is none.
pub fn existing(root: &Path, path: &RelativePath) -> Option<PathBuf> {
    let mut current = root.to_path_buf();
    for segment in path.segments() {
        current = child(&current, segment)?;
    }
    Some(current)
}

/// Where `path` lives or would be created below `root`: existing folders are matched by NFC name, the remaining
/// segments are used as given.
pub fn target(root: &Path, path: &RelativePath) -> PathBuf {
    let mut current = root.to_path_buf();
    let mut parent_exists = true;
    for segment in path.segments() {
        let found = if parent_exists { child(&current, segment) } else { None };
        parent_exists = found.is_some();
        current = found.unwrap_or_else(|| current.join(segment));
    }
    current
}

/// An entry in `directory` whose name equals `name` ignoring case and normalization.
///
/// # Errors
/// Fails when the directory cannot be listed.
pub fn clashing(directory: &Path, name: &str) -> io::Result<Option<PathBuf>> {
    if !directory.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if FileNamePolicy::is_same_name(&entry.file_name().to_string_lossy(), name) {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

/// The NFC form of an entry's name, as the catalog stores it.
pub fn name_of(entry: &Path) -> String {
    entry.file_name().map(|name| text::nfc(&name.to_string_lossy())).unwrap_or_default()
}

fn child(directory: &Path, name: &str) -> Option<PathBuf> {
    let direct = directory.join(name);
    if fs::symlink_metadata(&direct).is_ok() {
        return Some(direct);
    }
    fs::read_dir(directory).ok()?.filter_map(Result::ok).map(|entry| entry.path()).find(|entry| name_of(entry) == name)
}
