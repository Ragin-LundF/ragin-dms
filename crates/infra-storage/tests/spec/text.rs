use dms_application::port::storage::{StorageFailure, TextStore};
use dms_domain::document::DocumentId;
use dms_infra_storage::FileTextStore;

use crate::support::{failure_of, folder, read, write};

fn document_id() -> DocumentId {
    DocumentId::new("doc-1").expect("id")
}

#[test]
fn stored_text_is_read_back_and_named_by_the_document_id_in_the_subfolder_of_its_first_two_characters() {
    let temp = tempfile::tempdir().expect("temp dir");
    let text = "Page one\nStra\u{df}e, caf\u{e9}, \u{fc}ber";

    FileTextStore.write(&folder(temp.path()), &document_id(), text).expect("written");

    assert_eq!(FileTextStore.read(&folder(temp.path()), &document_id()).as_deref(), Some(text));
    assert_eq!(read(&temp.path().join(".dms-text/do/doc-1.txt")), text);
    assert!(!temp.path().join(".dms-text/doc-1.txt").exists());
}

#[test]
fn a_text_of_a_folder_not_moved_into_subfolders_yet_is_read_from_the_flat_file() {
    let temp = tempfile::tempdir().expect("temp dir");
    write(&temp.path().join(".dms-text/doc-1.txt"), "flat text");

    assert_eq!(FileTextStore.read(&folder(temp.path()), &document_id()).as_deref(), Some("flat text"));
}

#[test]
fn the_text_in_the_subfolder_wins_over_a_flat_one() {
    let temp = tempfile::tempdir().expect("temp dir");
    write(&temp.path().join(".dms-text/doc-1.txt"), "old flat text");

    FileTextStore.write(&folder(temp.path()), &document_id(), "new text").expect("written");

    assert_eq!(FileTextStore.read(&folder(temp.path()), &document_id()).as_deref(), Some("new text"));
}

#[test]
fn an_id_that_is_no_plain_file_name_is_never_written_outside_the_text_folder() {
    let temp = tempfile::tempdir().expect("temp dir");
    let library = temp.path().join("library");
    std::fs::create_dir(&library).expect("library");
    let escaping = DocumentId::new("../../escaped").expect("id");

    assert_eq!(failure_of(FileTextStore.write(&folder(&library), &escaping, "text")), Some(StorageFailure::IoError));
    assert!(!temp.path().join("escaped.txt").exists());
}

#[test]
fn missing_text_is_not_an_error() {
    let temp = tempfile::tempdir().expect("temp dir");

    assert_eq!(FileTextStore.read(&folder(temp.path()), &document_id()), None);
}

#[test]
fn an_offline_folder_fails_to_write_and_has_no_text() {
    let temp = tempfile::tempdir().expect("temp dir");
    let offline = folder(&temp.path().join("offline"));

    assert_eq!(
        failure_of(FileTextStore.write(&offline, &document_id(), "text")),
        Some(StorageFailure::FolderUnreachable)
    );
    assert_eq!(FileTextStore.read(&offline, &document_id()), None);
}
