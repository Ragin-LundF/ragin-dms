use std::collections::BTreeSet;
use std::path::Path;

use dms_application::port::sidecar::{SidecarFile, SidecarKind, SidecarLayout, SidecarMove};
use dms_application::port::storage::StorageFailure;
use dms_domain::document::DocumentId;
use dms_infra_storage::FileSidecarLayout;

use crate::support::{failure_of, folder, names_in, read, write};

fn text(id: &str) -> SidecarFile {
    SidecarFile { kind: SidecarKind::Text, document_id: DocumentId::new(id).expect("id") }
}

fn index(id: &str) -> SidecarFile {
    SidecarFile { kind: SidecarKind::Index, document_id: DocumentId::new(id).expect("id") }
}

fn flat_library(root: &Path) {
    write(&root.join(".dms-text/ab12.txt"), "text ab");
    write(&root.join(".dms-text/cd34.txt"), "text cd");
    write(&root.join(".dms-index/ab12.vec"), "vectors ab");
    write(&root.join(".dms-text/.dms-tmp-123"), "unfinished");
    write(&root.join(".dms-text/notes.md"), "not a text of the tool");
    write(&root.join(".dms-text/ab/ef56.txt"), "already in its subfolder");
}

#[test]
fn only_the_flat_texts_and_index_files_are_listed() {
    let temp = tempfile::tempdir().expect("temp dir");
    flat_library(temp.path());

    let files = FileSidecarLayout.flat_files(&folder(temp.path())).expect("listed");

    assert_eq!(files, vec![text("ab12"), text("cd34"), index("ab12")]);
}

#[test]
fn a_flat_file_moves_into_the_subfolder_of_its_first_two_characters() {
    let temp = tempfile::tempdir().expect("temp dir");
    flat_library(temp.path());
    let library = folder(temp.path());

    for file in [text("ab12"), index("ab12")] {
        assert_eq!(FileSidecarLayout.move_into_subfolder(&library, &file).expect("moved"), SidecarMove::Moved);
    }

    assert_eq!(read(&temp.path().join(".dms-text/ab/ab12.txt")), "text ab");
    assert_eq!(read(&temp.path().join(".dms-index/ab/ab12.vec")), "vectors ab");
    assert!(!temp.path().join(".dms-text/ab12.txt").exists());
    assert!(!temp.path().join(".dms-index/ab12.vec").exists());
}

#[test]
fn a_file_already_in_the_subfolder_is_never_replaced() {
    let temp = tempfile::tempdir().expect("temp dir");
    write(&temp.path().join(".dms-text/ab12.txt"), "flat");
    write(&temp.path().join(".dms-text/ab/ab12.txt"), "newer");
    write(&temp.path().join(".dms-text/cd34.txt"), "same");
    write(&temp.path().join(".dms-text/cd/cd34.txt"), "same");
    let library = folder(temp.path());

    let different = FileSidecarLayout.move_into_subfolder(&library, &text("ab12")).expect("kept");
    let identical = FileSidecarLayout.move_into_subfolder(&library, &text("cd34")).expect("kept");

    assert_eq!(different, SidecarMove::KeptExisting { identical: false });
    assert_eq!(identical, SidecarMove::KeptExisting { identical: true });
    assert_eq!(read(&temp.path().join(".dms-text/ab/ab12.txt")), "newer");
    assert_eq!(read(&temp.path().join(".dms-text/ab12.txt")), "flat");
    assert_eq!(read(&temp.path().join(".dms-text/cd34.txt")), "same");
}

#[test]
fn a_folder_is_current_only_once_it_is_marked() {
    let temp = tempfile::tempdir().expect("temp dir");
    let library = folder(temp.path());
    assert!(!FileSidecarLayout.is_current(&library));

    FileSidecarLayout.mark_current(&library).expect("marked");

    assert!(FileSidecarLayout.is_current(&library));
    assert_eq!(read(&temp.path().join(".dms-text/.layout")), "2\n");
    assert_eq!(names_in(&temp.path().join(".dms-text")), BTreeSet::from([".layout".to_owned()]));
    assert_eq!(FileSidecarLayout.flat_files(&library).expect("listed"), Vec::<SidecarFile>::new());
}

#[test]
fn a_folder_without_hidden_folders_has_no_flat_files() {
    let temp = tempfile::tempdir().expect("temp dir");

    assert_eq!(FileSidecarLayout.flat_files(&folder(temp.path())).expect("listed"), Vec::<SidecarFile>::new());
}

#[test]
fn an_offline_folder_is_not_current_and_cannot_be_listed() {
    let temp = tempfile::tempdir().expect("temp dir");
    let offline = folder(&temp.path().join("offline"));

    assert!(!FileSidecarLayout.is_current(&offline));
    assert_eq!(failure_of(FileSidecarLayout.flat_files(&offline)), Some(StorageFailure::FolderUnreachable));
    assert_eq!(failure_of(FileSidecarLayout.mark_current(&offline)), Some(StorageFailure::FolderUnreachable));
}
