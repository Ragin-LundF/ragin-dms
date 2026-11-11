use dms_domain::catalog::Catalog;
use dms_domain::change::{
    ChangeDetector, ContentConflict, DetectedChange, FileFacts, FolderSnapshot, MissingFile, ModifiedFile, MovedFile,
    NewFile,
};
use dms_domain::document::Document;
use dms_domain::library::LibraryFolderId;
use indexmap::{indexmap, indexset};

use crate::test_data::{self, hash, path};

fn main_folder() -> LibraryFolderId {
    LibraryFolderId::new("main").expect("valid")
}

fn backup() -> LibraryFolderId {
    LibraryFolderId::new("backup").expect("valid")
}

fn report() -> Document {
    test_data::document_with("d1", "projects/report.pdf", 1, 10)
}

fn catalog() -> Catalog {
    test_data::catalog_with(&[report()])
}

fn snapshot_of(folder_id: LibraryFolderId, documents: &[Document]) -> FolderSnapshot {
    FolderSnapshot {
        folder_id,
        files: documents.iter().map(|document| (document.path.clone(), facts(document))).collect(),
    }
}

fn facts(document: &Document) -> FileFacts {
    FileFacts { size: document.size, content_hash: Some(document.content_hash.clone()) }
}

#[test]
fn folders_matching_the_catalog_produce_no_changes() {
    let changes = ChangeDetector::detect(
        &catalog(),
        &[snapshot_of(main_folder(), &[report()]), snapshot_of(backup(), &[report()])],
    );

    assert_eq!(changes, [] as [dms_domain::change::DetectedChange; 0]);
}

#[test]
fn an_unhashed_file_with_the_catalogued_size_counts_as_unchanged() {
    let snapshot = FolderSnapshot {
        folder_id: main_folder(),
        files: indexmap! { report().path => FileFacts { size: 10, content_hash: None } },
    };

    assert_eq!(ChangeDetector::detect(&catalog(), &[snapshot]), [] as [dms_domain::change::DetectedChange; 0]);
}

#[test]
fn different_content_in_one_folder_is_a_modification_of_that_folder() {
    let snapshot = FolderSnapshot {
        folder_id: backup(),
        files: indexmap! { report().path => FileFacts { size: 11, content_hash: Some(hash(2)) } },
    };

    let changes = ChangeDetector::detect(&catalog(), &[snapshot]);

    assert_eq!(
        changes,
        vec![DetectedChange::ModifiedFile(ModifiedFile {
            document_id: report().id,
            path: report().path,
            folder_id: backup(),
            size: 11,
            content_hash: Some(hash(2)),
        })]
    );
}

#[test]
fn a_file_found_under_another_path_with_the_same_content_is_a_move() {
    let moved = path("archive/report.pdf");
    let snapshot = FolderSnapshot {
        folder_id: main_folder(),
        files: indexmap! { moved.clone() => FileFacts { size: 10, content_hash: Some(report().content_hash) } },
    };

    let changes = ChangeDetector::detect(&catalog(), &[snapshot]);

    assert_eq!(
        changes,
        vec![DetectedChange::MovedFile(MovedFile {
            document_id: report().id,
            path: moved,
            previous_path: report().path,
            folder_id: main_folder(),
        })]
    );
}

#[test]
fn a_missing_file_names_the_folders_that_lack_it_and_the_ones_that_still_have_it() {
    let changes =
        ChangeDetector::detect(&catalog(), &[snapshot_of(main_folder(), &[]), snapshot_of(backup(), &[report()])]);

    assert_eq!(
        changes,
        vec![DetectedChange::MissingFile(MissingFile {
            document_id: report().id,
            path: report().path,
            missing_in: indexset! { main_folder() },
            present_in: indexset! { backup() },
        })]
    );
}

#[test]
fn an_unreachable_folder_is_not_reported_as_missing_anything() {
    let changes = ChangeDetector::detect(&catalog(), &[snapshot_of(main_folder(), &[report()])]);

    assert_eq!(changes, [] as [dms_domain::change::DetectedChange; 0]);
}

#[test]
fn an_uncatalogued_file_is_new_and_lists_every_folder_that_has_it() {
    let extra = test_data::document_with("x", "inbox/scan.pdf", 3, 5);

    let changes = ChangeDetector::detect(
        &catalog(),
        &[snapshot_of(main_folder(), &[report(), extra.clone()]), snapshot_of(backup(), &[report(), extra.clone()])],
    );

    assert_eq!(
        changes,
        vec![DetectedChange::NewFile(NewFile {
            path: extra.path,
            present_in: indexset! { main_folder(), backup() },
            size: 5,
            content_hash: Some(extra.content_hash),
        })]
    );
}

#[test]
fn an_uncatalogued_file_with_different_content_per_folder_is_a_conflict() {
    let scan = path("inbox/scan.pdf");
    let snapshots = [
        FolderSnapshot {
            folder_id: main_folder(),
            files: indexmap! { report().path => facts(&report()), scan.clone() => FileFacts { size: 5, content_hash: Some(hash(4)) } },
        },
        FolderSnapshot {
            folder_id: backup(),
            files: indexmap! { report().path => facts(&report()), scan.clone() => FileFacts { size: 6, content_hash: Some(hash(5)) } },
        },
    ];

    let changes = ChangeDetector::detect(&catalog(), &snapshots);

    assert_eq!(
        changes,
        vec![DetectedChange::ContentConflict(ContentConflict {
            path: scan,
            content_hashes: indexmap! { main_folder() => hash(4), backup() => hash(5) },
        })]
    );
}

#[test]
fn changes_are_sorted_by_path() {
    let first = test_data::document_with("a", "a.pdf", 10, 100);
    let last = test_data::document_with("z", "z.pdf", 11, 100);

    let changes = ChangeDetector::detect(&catalog(), &[snapshot_of(main_folder(), &[last, first])]);

    assert_eq!(
        changes.iter().map(|change| change.path().value()).collect::<Vec<_>>(),
        vec!["a.pdf", "projects/report.pdf", "z.pdf"]
    );
}
