use std::path::Path;

/// Moves a file or folder of a library folder to a trash, from where the user can restore it.
pub trait SystemTrash: Send + Sync {
    /// `false` if `entry` could not be moved; nothing was deleted then.
    fn move_to_trash(&self, library_root: &Path, entry: &Path) -> bool;
}
