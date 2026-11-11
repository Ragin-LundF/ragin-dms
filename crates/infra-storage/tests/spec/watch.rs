use std::time::Duration;

use dms_application::port::storage::FolderWatcher;
use dms_infra_storage::FileSystemFolderWatcher;

use crate::support::{folder, path, write};

#[test]
fn a_file_dropped_into_the_watched_folder_is_reported_and_a_missing_folder_is_left_out() {
    let temp = tempfile::tempdir().expect("temp dir");
    let library = temp.path().join("library");
    std::fs::create_dir_all(library.join("Inbox")).expect("inbox");
    let without_inbox = temp.path().join("other");
    std::fs::create_dir_all(&without_inbox).expect("folder");

    let changes = FileSystemFolderWatcher.changes(&[folder(&library), folder(&without_inbox)], &path("Inbox"));
    std::thread::sleep(Duration::from_millis(300));
    write(&library.join("Inbox/scan.pdf"), "alpha");

    assert!(changes.recv_timeout(Duration::from_secs(10)).is_ok());
}
