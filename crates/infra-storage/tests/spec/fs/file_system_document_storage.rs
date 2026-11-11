use std::fs::{self, FileTimes};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dms_application::port::storage::{DocumentStorage, ExternalFile, StorageFailure};
use dms_domain::library::RelativePath;
use dms_infra_storage::FileSystemDocumentStorage;

use super::Folders;
use crate::support::{RecordingTrash, failure_of, folder, hash_of, names_in, path, read, value_of, write};

#[test]
fn an_imported_file_is_copied_with_its_hash_and_modification_time() {
    let folders = Folders::new();
    let source = write(&folders.temp.path().join("outside/scan.pdf"), "alpha");
    #[allow(clippy::duration_suboptimal_units)] // a Unix timestamp reads best in seconds
    let modified = UNIX_EPOCH + Duration::from_secs(1_685_620_800);
    fs::File::options()
        .write(true)
        .open(&source)
        .expect("source")
        .set_times(FileTimes::new().set_modified(modified))
        .expect("time");

    let facts = value_of(folders.storage.import_file(
        &folders.primary,
        &ExternalFile::new(source.to_string_lossy()).expect("location"),
        &path("letters/scan.pdf"),
    ));

    assert_eq!(facts.content_hash, Some(hash_of("alpha")));
    let copy = folders.primary_root.join("letters/scan.pdf");
    assert_eq!(read(&copy), "alpha");
    assert_eq!(fs::metadata(&copy).expect("copy").modified().expect("time"), modified);
    assert_eq!(names_in(copy.parent().expect("parent")), ["scan.pdf".to_owned()].into());
}

#[test]
fn an_import_never_takes_a_name_that_differs_only_in_case() {
    let folders = Folders::new();
    write(&folders.primary_root.join("Note.pdf"), "kept");
    let source = write(&folders.temp.path().join("outside/note.pdf"), "new");

    let result = folders.storage.import_file(
        &folders.primary,
        &ExternalFile::new(source.to_string_lossy()).expect("location"),
        &path("note.pdf"),
    );

    assert_eq!(failure_of(result), Some(StorageFailure::AlreadyExists));
    assert_eq!(read(&folders.primary_root.join("Note.pdf")), "kept");
}

#[test]
fn an_import_of_a_missing_source_fails_with_not_found() {
    let folders = Folders::new();
    let missing = folders.temp.path().join("outside/missing.pdf");

    let result = folders.storage.import_file(
        &folders.primary,
        &ExternalFile::new(missing.to_string_lossy()).expect("location"),
        &path("missing.pdf"),
    );

    assert_eq!(failure_of(result), Some(StorageFailure::NotFound));
}

#[test]
fn a_copy_replaces_an_existing_file_only_when_asked_and_only_after_trashing_it() {
    let folders = Folders::new();
    write(&folders.primary_root.join("a.pdf"), "new");
    write(&folders.sync_root.join("a.pdf"), "old");

    let refused = folders.storage.copy_file(&folders.primary, &folders.sync, &path("a.pdf"), false);
    let replaced = folders.storage.copy_file(&folders.primary, &folders.sync, &path("a.pdf"), true);

    assert_eq!(failure_of(refused), Some(StorageFailure::AlreadyExists));
    assert!(replaced.is_ok());
    assert_eq!(read(&folders.sync_root.join("a.pdf")), "new");
    assert_eq!(folders.trash.content_of("a.pdf").as_deref(), Some("old"));
}

#[test]
fn a_copy_keeps_the_existing_file_when_the_trash_refuses_it() {
    let folders = Folders::new();
    let refusing = FileSystemDocumentStorage::new(Arc::new(RecordingTrash::refusing(folders.temp.path())));
    write(&folders.primary_root.join("a.pdf"), "new");
    write(&folders.sync_root.join("a.pdf"), "old");

    let result = refusing.copy_file(&folders.primary, &folders.sync, &path("a.pdf"), true);

    assert_eq!(failure_of(result), Some(StorageFailure::IoError));
    assert_eq!(read(&folders.sync_root.join("a.pdf")), "old");
}

#[test]
fn a_file_stored_with_a_decomposed_name_is_found_by_its_composed_path() {
    let folders = Folders::new();
    write(&folders.primary_root.join("cafe\u{301}/menu.pdf"), "alpha");

    let facts = folders.storage.read_facts(&folders.primary, &path("caf\u{e9}/menu.pdf"));
    let names = folders.storage.list_names(&folders.primary, &RelativePath::root());

    assert!(facts.is_ok());
    assert_eq!(value_of(names), ["caf\u{e9}"]);
}

#[test]
fn listing_a_missing_directory_fails_with_not_found() {
    let folders = Folders::new();

    assert_eq!(
        failure_of(folders.storage.list_names(&folders.primary, &path("nowhere"))),
        Some(StorageFailure::NotFound)
    );
}

#[test]
fn a_folder_that_is_not_reachable_is_reported_as_such() {
    let folders = Folders::new();
    let offline = folder(&folders.temp.path().join("offline"));

    let result = folders.storage.list_names(&offline, &RelativePath::root());

    assert_eq!(failure_of(result), Some(StorageFailure::FolderUnreachable));
    assert!(!folders.temp.path().join("offline").exists());
}

#[test]
fn two_files_count_as_the_same_content_only_if_every_byte_is_the_same() {
    let folders = Folders::new();
    write(&folders.primary_root.join("a.pdf"), "alpha-1");
    write(&folders.primary_root.join("copy/a.pdf"), "alpha-1");
    write(&folders.primary_root.join("b.pdf"), "alpha-2");

    let copy = folders.storage.same_content(&folders.primary, &path("a.pdf"), &path("copy/a.pdf"));
    let same_size = folders.storage.same_content(&folders.primary, &path("a.pdf"), &path("b.pdf"));

    assert!(value_of(copy));
    assert!(!value_of(same_size));
}

#[test]
fn a_file_is_never_a_copy_of_itself() {
    let folders = Folders::new();
    write(&folders.primary_root.join("a.pdf"), "alpha");

    assert!(!value_of(folders.storage.same_content(&folders.primary, &path("a.pdf"), &path("a.pdf"))));
}

#[test]
fn a_replacement_takes_the_place_of_the_old_file_which_goes_to_the_trash() {
    let folders = Folders::new();
    write(&folders.primary_root.join("letters/a.pdf"), "old");
    write(&folders.primary_root.join("letters/.dms-tmp-1.pdf"), "edited");

    let facts = value_of(folders.storage.replace_with(
        &folders.primary,
        &path("letters/a.pdf"),
        &path("letters/.dms-tmp-1.pdf"),
    ));

    assert_eq!(facts.content_hash, Some(hash_of("edited")));
    assert_eq!(read(&folders.primary_root.join("letters/a.pdf")), "edited");
    assert_eq!(folders.trash.content_of("a.pdf").as_deref(), Some("old"));
    assert_eq!(names_in(&folders.primary_root.join("letters")), ["a.pdf".to_owned()].into());
}

#[test]
fn the_creation_day_of_a_file_is_known_and_a_missing_file_has_none() {
    let folders = Folders::new();
    write(&folders.primary_root.join("a.pdf"), "alpha");

    let created = folders.storage.created_on(&folders.primary, &path("a.pdf"));
    let missing = folders.storage.created_on(&folders.primary, &path("missing.pdf"));

    if fs::metadata(folders.primary_root.join("a.pdf")).expect("file").created().is_ok() {
        let today: chrono::DateTime<chrono::Local> = SystemTime::now().into();
        assert_eq!(created, Some(today.date_naive()));
    }
    assert_eq!(missing, None);
}
