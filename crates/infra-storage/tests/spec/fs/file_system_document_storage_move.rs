use dms_application::port::storage::{DocumentStorage, StorageFailure};
use dms_domain::library::RelativePath;

use super::Folders;
use crate::support::{failure_of, hash_of, names_in, path, read, value_of, write};

#[test]
fn a_move_creates_missing_parent_folders() {
    let folders = Folders::new();
    write(&folders.primary_root.join("a.pdf"), "alpha");

    folders.storage.move_entry(&folders.primary, &path("a.pdf"), &path("archive/2024/a.pdf")).expect("moved");

    assert_eq!(read(&folders.primary_root.join("archive/2024/a.pdf")), "alpha");
    assert_eq!(names_in(&folders.primary_root), ["archive".to_owned()].into());
}

#[test]
fn a_rename_that_only_changes_the_case_works() {
    let folders = Folders::new();
    write(&folders.primary_root.join("note.pdf"), "alpha");

    folders.storage.move_entry(&folders.primary, &path("note.pdf"), &path("Note.pdf")).expect("renamed");

    assert_eq!(names_in(&folders.primary_root), ["Note.pdf".to_owned()].into());
    assert_eq!(read(&folders.primary_root.join("Note.pdf")), "alpha");
}

#[test]
fn a_move_onto_a_name_that_differs_only_in_case_fails() {
    let folders = Folders::new();
    write(&folders.primary_root.join("a.pdf"), "alpha");
    write(&folders.primary_root.join("b.pdf"), "beta");

    let result = folders.storage.move_entry(&folders.primary, &path("a.pdf"), &path("B.pdf"));

    assert_eq!(failure_of(result), Some(StorageFailure::AlreadyExists));
    assert_eq!(read(&folders.primary_root.join("b.pdf")), "beta");
}

#[test]
fn a_move_of_a_missing_entry_fails_with_not_found() {
    let folders = Folders::new();

    let result = folders.storage.move_entry(&folders.primary, &path("missing.pdf"), &path("other.pdf"));

    assert_eq!(failure_of(result), Some(StorageFailure::NotFound));
}

#[test]
fn a_trashed_file_goes_to_the_system_trash() {
    let folders = Folders::new();
    write(&folders.primary_root.join("letters/a.pdf"), "alpha");

    folders.storage.move_to_trash(&folders.primary, &path("letters/a.pdf")).expect("trashed");

    assert_eq!(folders.trash.content_of("a.pdf").as_deref(), Some("alpha"));
    assert!(names_in(&folders.primary_root.join("letters")).is_empty());
    assert_eq!(
        failure_of(folders.storage.move_to_trash(&folders.primary, &path("letters/a.pdf"))),
        Some(StorageFailure::NotFound)
    );
}

#[test]
fn creating_an_existing_folder_succeeds_but_a_clash_with_another_entry_does_not() {
    let folders = Folders::new();
    write(&folders.primary_root.join("notes"), "a file");

    let first = folders.storage.create_folder(&folders.primary, &path("letters"));
    let again = folders.storage.create_folder(&folders.primary, &path("letters"));
    let other_case = folders.storage.create_folder(&folders.primary, &path("Letters"));
    let on_file = folders.storage.create_folder(&folders.primary, &path("notes"));

    assert!(first.is_ok());
    assert!(again.is_ok());
    assert_eq!(failure_of(other_case), Some(StorageFailure::AlreadyExists));
    assert_eq!(failure_of(on_file), Some(StorageFailure::AlreadyExists));
}

#[test]
fn the_facts_of_a_file_are_always_hashed() {
    let folders = Folders::new();
    write(&folders.primary_root.join("a.pdf"), "alpha");

    let facts = value_of(folders.storage.read_facts(&folders.primary, &path("a.pdf")));

    assert_eq!(facts.size, 5);
    assert_eq!(facts.content_hash, Some(hash_of("alpha")));
    assert_eq!(
        failure_of(folders.storage.read_facts(&folders.primary, &RelativePath::root())),
        Some(StorageFailure::NotFound)
    );
}
