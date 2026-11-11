//! Tells the UI that a folder below the library folders changed on disk, so it can scan.

mod file_system_folder_watcher;

pub use file_system_folder_watcher::FileSystemFolderWatcher;
