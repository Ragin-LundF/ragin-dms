use std::fs;

use dms_application::port::storage::{FolderOperation, OperationJournal, PendingOperation};
use dms_domain::document::DocumentId;
use dms_domain::library::LibraryFolderId;
use dms_infra_storage::FileOperationJournal;

use crate::catalog::instant;
use crate::support::{fixture, path, read};

fn pending(id: &str, operation: FolderOperation) -> PendingOperation {
    PendingOperation {
        id: id.to_owned(),
        folder_id: LibraryFolderId::new("sync").expect("id"),
        operation,
        recorded_at: instant("2024-05-01T08:00:00Z"),
    }
}

fn every_operation() -> Vec<PendingOperation> {
    [
        FolderOperation::CopyFromPrimary { path: path("a.pdf"), replace_existing: true },
        FolderOperation::CopyFromPrimary { path: path("caf\u{e9}/b.pdf"), replace_existing: false },
        FolderOperation::CreateFolder(path("letters")),
        FolderOperation::Move { from: path("a.pdf"), to: path("x/a.pdf") },
        FolderOperation::MoveToTrash(path("old.pdf")),
        FolderOperation::CopyText(DocumentId::new("doc-1").expect("id")),
        FolderOperation::CopyPlan("20261022-080000".to_owned()),
        FolderOperation::CopyIndex(DocumentId::new("doc-2").expect("id")),
        FolderOperation::SaveCatalog,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, operation)| pending(&format!("op-{index}"), operation))
    .collect()
}

#[test]
fn every_kind_of_operation_survives_a_restart_in_the_format_jackson_wrote() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("journal.yaml");
    let journal = FileOperationJournal::new(&path);
    let operations = every_operation();

    for operation in &operations {
        journal.record(operation).expect("recorded");
    }

    assert_eq!(FileOperationJournal::new(&path).pending().expect("pending"), operations);
    assert_eq!(read(&path), fixture("jackson/journal.yaml"));
}

#[test]
fn a_removed_operation_is_gone() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("journal.yaml");
    let journal = FileOperationJournal::new(&path);
    let first = pending("op-1", FolderOperation::SaveCatalog);
    let second = pending("op-2", FolderOperation::CreateFolder(crate::support::path("a")));
    journal.record(&first).expect("recorded");
    journal.record(&second).expect("recorded");

    journal.remove("op-1").expect("removed");

    assert_eq!(FileOperationJournal::new(&path).pending().expect("pending"), [second]);
}

#[test]
fn an_operation_of_a_newer_version_is_skipped() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("journal.yaml");
    fs::write(
        &path,
        "operations:\n- id: op-1\n  folderId: sync\n  recordedAt: 2024-05-01T08:00:00Z\n  type: FUTURE_OPERATION\n",
    )
    .expect("file");

    assert_eq!(FileOperationJournal::new(&path).pending().expect("pending"), []);
}
