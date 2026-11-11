use dms_application::port::storage::{StorageFailure, TextStore};
use dms_domain::document::DocumentId;
use dms_infra_storage::FileTextStore;

use crate::support::{failure_of, folder, read};

fn document_id() -> DocumentId {
    DocumentId::new("doc-1").expect("id")
}

#[test]
fn stored_text_is_read_back_and_named_by_the_document_id() {
    let temp = tempfile::tempdir().expect("temp dir");
    let text = "Page one\nStra\u{df}e, caf\u{e9}, \u{fc}ber";

    FileTextStore.write(&folder(temp.path()), &document_id(), text).expect("written");

    assert_eq!(FileTextStore.read(&folder(temp.path()), &document_id()).as_deref(), Some(text));
    assert_eq!(read(&temp.path().join(".dms-text/doc-1.txt")), text);
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
