//! The archive health state of this device: `health.yaml`, the catalog snapshots, the archive check's reads that
//! never download a placeholder, and the schedule in `settings.yaml`.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use dms_application::port::health::{CatalogSnapshotId, CatalogSnapshots, FolderCheck, HealthReport, HealthStore};
use dms_application::port::state::{Preferences, SettingsRepository};
use dms_application::port::storage::{DocumentStorage, StorageFailure};
use dms_domain::catalog::{Catalog, LibraryId};
use dms_domain::document::{Document, DocumentId};
use dms_domain::health::{HealthFinding, HealthSchedule};
use dms_domain::library::LibraryFolderId;
use dms_infra_storage::{FileCatalogSnapshots, FileHealthStore, FileSettingsRepository, FileSystemDocumentStorage};

use crate::support::{RecordingTrash, failure_of, folder, hash_of, path, read, write};

fn at(text: &str) -> DateTime<Utc> {
    text.parse().expect("instant")
}

fn library_id() -> LibraryId {
    LibraryId::new("library-1").expect("id")
}

fn catalog_with(documents: &[&str]) -> Catalog {
    documents.iter().fold(Catalog::empty(library_id()), |catalog, name| {
        let document = Document::new(
            DocumentId::new(format!("id-{name}")).expect("id"),
            path(name),
            hash_of(name),
            4,
            at("2024-01-01T10:00:00Z"),
        )
        .expect("document");
        match catalog.add_document(document).expect("valid") {
            dms_domain::catalog::CatalogUpdate::Applied(catalog) => catalog,
            dms_domain::catalog::CatalogUpdate::Rejected(reason) => panic!("{reason:?}"),
        }
    })
}

#[test]
fn the_health_report_survives_a_restart() {
    let temp = tempfile::tempdir().expect("temp dir");
    let file = temp.path().join("health.yaml");
    let folder_id = LibraryFolderId::new("sync").expect("id");
    let report = HealthReport {
        library_id: library_id(),
        started_at: Some(at("2026-11-11T03:00:00Z")),
        finished_at: Some(at("2026-11-11T03:12:30Z")),
        resume_after: Some(DocumentId::new("id-b").expect("id")),
        folders: vec![FolderCheck {
            checked: 12,
            not_downloaded: 2,
            missing: 1,
            offline: true,
            ..FolderCheck::new(folder_id.clone())
        }],
        findings: vec![
            HealthFinding::Damaged {
                document_id: DocumentId::new("id-a").expect("id"),
                folder_id: folder_id.clone(),
                source_folder_id: LibraryFolderId::new("primary").expect("id"),
            },
            HealthFinding::Unreadable { document_id: DocumentId::new("id-c").expect("id") },
        ],
    };

    assert_eq!(FileHealthStore::new(&file).load().expect("read"), None);
    FileHealthStore::new(&file).save(&report).expect("saved");

    assert_eq!(FileHealthStore::new(&file).load().expect("read"), Some(report));
    assert!(read(&file).contains("kind: damaged"));
}

#[test]
fn snapshots_are_listed_newest_first_and_load_back_as_saved() {
    let temp = tempfile::tempdir().expect("temp dir");
    let snapshots = FileCatalogSnapshots::new(temp.path().join("snapshots"));
    let first = catalog_with(&["a.pdf"]);
    let second = catalog_with(&["a.pdf", "b.pdf"]);

    snapshots.save(&first, at("2026-11-10T08:00:00Z")).expect("saved");
    snapshots.save(&second, at("2026-11-11T09:14:00Z")).expect("saved");
    // A leftover file in the folder is no snapshot.
    write(&temp.path().join("snapshots/library-1/notes.txt"), "x");

    let listed = snapshots.list(&library_id()).expect("listed");
    let ids: Vec<&str> = listed.iter().map(|snapshot| snapshot.id.value()).collect();
    assert_eq!(
        ids,
        vec![
            format!("20261111T091400Z-r{}", second.revision()).as_str(),
            format!("20261110T080000Z-r{}", first.revision()).as_str(),
        ]
    );
    assert_eq!(listed[0].taken_at, at("2026-11-11T09:14:00Z"));
    assert_eq!(snapshots.load(&library_id(), &listed[1].id).expect("loaded"), first);

    snapshots.remove(&library_id(), &listed[1].id).expect("removed");
    assert_eq!(snapshots.list(&library_id()).expect("listed").len(), 1);
}

#[test]
fn a_library_without_snapshots_has_none_and_an_unknown_one_is_not_found() {
    let temp = tempfile::tempdir().expect("temp dir");
    let snapshots = FileCatalogSnapshots::new(temp.path().join("snapshots"));
    let unknown = CatalogSnapshotId::new("20200101T000000Z-r1").expect("id");

    assert_eq!(snapshots.list(&library_id()).expect("listed"), Vec::new());
    assert_eq!(failure_of(snapshots.load(&library_id(), &unknown)), Some(StorageFailure::NotFound));
}

#[test]
fn a_library_id_that_is_no_plain_name_never_becomes_a_path() {
    let temp = tempfile::tempdir().expect("temp dir");
    let snapshots = FileCatalogSnapshots::new(temp.path().join("snapshots"));
    let outside = Catalog::empty(LibraryId::new("../outside").expect("id"));

    assert_eq!(
        failure_of(snapshots.save(&outside, at("2026-11-11T09:14:00Z"))),
        Some(StorageFailure::UnsupportedFormat)
    );
    assert!(!temp.path().join("outside").exists());
}

#[test]
fn a_local_file_is_read_and_an_icloud_stub_is_reported_as_not_downloaded() {
    let temp = tempfile::tempdir().expect("temp dir");
    let storage = FileSystemDocumentStorage::new(Arc::new(RecordingTrash::new(&temp.path().join("trash"))));
    let library = folder(&temp.path().join("library"));
    write(&temp.path().join("library/docs/a.pdf"), "abcd");
    write(&temp.path().join("library/docs/.b.pdf.icloud"), "stub");

    let local = storage.read_local_facts(&library, &path("docs/a.pdf")).expect("read");
    assert_eq!(local.and_then(|facts| facts.content_hash), Some(hash_of("abcd")));
    assert_eq!(storage.read_local_facts(&library, &path("docs/b.pdf")).expect("stub"), None);
    assert_eq!(failure_of(storage.read_local_facts(&library, &path("docs/c.pdf"))), Some(StorageFailure::NotFound));
}

#[test]
fn the_check_schedule_is_saved_only_when_it_is_not_the_default() {
    let temp = tempfile::tempdir().expect("temp dir");
    let file = temp.path().join("settings.yaml");
    let repository = FileSettingsRepository::new(&file);

    repository.save_preferences(&Preferences::default()).expect("saved");
    assert!(!read(&file).contains("healthCheck"));

    let monthly = Preferences { health_check: HealthSchedule::Monthly, ..Preferences::default() };
    repository.save_preferences(&monthly).expect("saved");
    assert!(read(&file).contains("healthCheck: monthly"));
    assert_eq!(FileSettingsRepository::new(&file).load_preferences().expect("read"), monthly);
}
