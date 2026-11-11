use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::fs::atomic_files;
use crate::platform::SystemTrash;

/// The system trash (Finder, Explorer, desktop environment). Where the system has none, for example some Linux
/// desktops or network shares, the entry goes to `.dms-trash/<epoch millis>/<relative path>` inside the library
/// folder instead. Nothing is ever deleted permanently.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlatformTrash;

impl PlatformTrash {
    pub const FALLBACK_FOLDER: &'static str = ".dms-trash";
}

impl SystemTrash for PlatformTrash {
    fn move_to_trash(&self, library_root: &Path, entry: &Path) -> bool {
        // Some drives report success without a trash; only an entry that is really gone counts.
        if trash::delete(entry).is_ok() && fs::symlink_metadata(entry).is_err() {
            return true;
        }
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis());
        move_to_fallback(library_root, entry, millis)
    }
}

/// Moves `entry` to `.dms-trash/<millis>/<path relative to library_root>`, never over an existing entry.
pub fn move_to_fallback(library_root: &Path, entry: &Path, millis: u128) -> bool {
    let Ok(relative) = entry.strip_prefix(library_root) else { return false };
    let target = library_root.join(PlatformTrash::FALLBACK_FOLDER).join(millis.to_string()).join(relative);
    target.parent().is_some_and(|parent| fs::create_dir_all(parent).is_ok())
        && atomic_files::rename_new(entry, &target).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fallback_keeps_the_relative_path_below_a_time_folder() {
        let root = tempfile::tempdir().expect("temp dir");
        let entry = root.path().join("letters/a.pdf");
        fs::create_dir_all(entry.parent().expect("parent")).expect("folder");
        fs::write(&entry, "alpha").expect("file");

        assert!(move_to_fallback(root.path(), &entry, 1_700_000_000_000));

        let trashed = root.path().join(".dms-trash/1700000000000/letters/a.pdf");
        assert_eq!(fs::read_to_string(trashed).expect("trashed file"), "alpha");
        assert!(!entry.exists());
    }

    #[test]
    fn the_fallback_refuses_an_entry_outside_the_library() {
        let root = tempfile::tempdir().expect("temp dir");
        let outside = tempfile::NamedTempFile::new().expect("temp file");

        assert!(!move_to_fallback(root.path(), outside.path(), 1));
        assert!(outside.path().exists());
    }
}
