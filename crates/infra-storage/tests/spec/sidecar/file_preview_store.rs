use std::collections::BTreeSet;

use dms_application::port::sidecar::PreviewStore;
use dms_application::port::storage::StorageFailure;
use dms_domain::document::DocumentId;
use dms_infra_storage::FilePreviewStore;

use crate::support::{failure_of, folder, hash_of, names_in, write};

const ID: &str = "3f2504e0-4f89-11d3-9a0c-0305e82c3301";

fn document_id() -> DocumentId {
    DocumentId::new(ID).expect("id")
}

#[test]
fn a_preview_is_stored_beside_the_text_named_by_id_and_hash_and_read_back_for_that_hash() {
    let temp = tempfile::tempdir().expect("temp dir");
    let hash = hash_of("version one");

    FilePreviewStore.write(&folder(temp.path()), &document_id(), &hash, &[1, 2, 3]).expect("written");

    let name = format!("{ID}.{}.jpg", &hash.value()[..12]);
    assert_eq!(names_in(&temp.path().join(".dms-text/3f")), BTreeSet::from([name]));
    assert_eq!(FilePreviewStore.read(&folder(temp.path()), &document_id(), &hash), Some(vec![1, 2, 3]));
    assert_eq!(FilePreviewStore.read(&folder(temp.path()), &document_id(), &hash_of("version two")), None);
}

#[test]
fn a_preview_for_a_new_hash_removes_the_older_ones_of_the_document_only() {
    let temp = tempfile::tempdir().expect("temp dir");
    let shard = temp.path().join(".dms-text/3f");
    let neighbour = format!("3f000000-0000-0000-0000-000000000000.{}.jpg", &hash_of("other").value()[..12]);
    write(&shard.join(&neighbour), "other document");
    write(&shard.join(format!("{ID}.txt")), "the text");
    FilePreviewStore.write(&folder(temp.path()), &document_id(), &hash_of("version one"), &[1]).expect("first");

    let newer = hash_of("version two");
    FilePreviewStore.write(&folder(temp.path()), &document_id(), &newer, &[2]).expect("second");

    let current = format!("{ID}.{}.jpg", &newer.value()[..12]);
    assert_eq!(names_in(&shard), BTreeSet::from([current, neighbour, format!("{ID}.txt")]));
    assert_eq!(FilePreviewStore.read(&folder(temp.path()), &document_id(), &newer), Some(vec![2]));
}

#[test]
fn an_offline_folder_fails_to_store_and_has_no_preview() {
    let temp = tempfile::tempdir().expect("temp dir");
    let offline = folder(&temp.path().join("offline"));
    let hash = hash_of("content");

    assert_eq!(
        failure_of(FilePreviewStore.write(&offline, &document_id(), &hash, &[1])),
        Some(StorageFailure::FolderUnreachable)
    );
    assert_eq!(FilePreviewStore.read(&offline, &document_id(), &hash), None);
}
