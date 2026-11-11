//! `FileSystemLibraryScannerTest`, plus the scan cache against the Jackson golden file.

use std::fs::{self, FileTimes};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use dms_application::port::storage::{LibraryScanner, NoScanListener, StorageFailure};
use dms_domain::catalog::{Catalog, LibraryId};
use dms_domain::change::FileFacts;
use dms_domain::library::RelativePath;
use dms_infra_storage::FileSystemLibraryScanner;
use indexmap::{IndexMap, IndexSet};
use tempfile::TempDir;

use crate::catalog::document;
use crate::support::{Applied, RecordingListener, failure_of, fixture, folder, hash_of, path, read, value_of, write};

struct Scan {
    temp: TempDir,
    root: PathBuf,
    scanner: FileSystemLibraryScanner,
}

impl Scan {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path().join("library");
        fs::create_dir_all(&root).expect("library");
        let scanner = FileSystemLibraryScanner::new(temp.path().join("cache"));
        Self { temp, root, scanner }
    }

    fn files(&self, catalog: &Catalog) -> IndexMap<RelativePath, FileFacts> {
        value_of(self.scanner.scan(&folder(&self.root), catalog, &NoScanListener)).files
    }
}

fn empty() -> Catalog {
    Catalog::empty(LibraryId::new("library-1").expect("id"))
}

fn catalog_with(raw_path: &str, content: &str) -> Catalog {
    empty().add_document(document("doc-1", raw_path, content)).applied()
}

fn set_modified(file: &Path, seconds: u64) {
    let times = FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds));
    fs::File::options().write(true).open(file).expect("file").set_times(times).expect("time");
}

#[test]
fn system_clutter_and_the_tools_own_files_are_not_listed() {
    let scan = Scan::new();
    for name in [
        ".DS_Store",
        "._a.pdf",
        "Thumbs.db",
        "dms-catalog.yaml",
        "dms-catalog 2.yaml",
        ".dms-text/doc-1.txt",
        ".dms-text/.layout",
        ".dms-text/do/doc-1.txt",
        ".dms-text/do/doc-1.0123456789ab.jpg",
        ".dms-index/do/doc-1.vec",
        "letters/.dms-text/do/doc-2.txt",
        ".dms-plans/20261022-080000.yaml",
        ".dms-index/doc-1.vec",
        ".dms-trash/1/old.pdf",
        "letters/.dms-tmp-123",
        "letters/a.pdf",
    ] {
        write(&scan.root.join(name), "x");
    }

    let files = scan.files(&empty());

    assert_eq!(files.keys().cloned().collect::<IndexSet<_>>(), IndexSet::from([path("letters/a.pdf")]));
}

#[test]
fn files_the_catalog_does_not_know_are_hashed() {
    let scan = Scan::new();
    write(&scan.root.join("a.pdf"), "alpha");

    let files = scan.files(&empty());

    assert_eq!(files[&path("a.pdf")], FileFacts { size: 5, content_hash: Some(hash_of("alpha")) });
}

#[test]
fn a_known_file_of_the_catalogued_size_is_not_hashed() {
    let scan = Scan::new();
    // Same size, different content: only hashing would notice.
    write(&scan.root.join("a.pdf"), "ALPHA");

    let files = scan.files(&catalog_with("a.pdf", "alpha"));

    assert_eq!(files[&path("a.pdf")], FileFacts { size: 5, content_hash: None });
}

#[test]
fn a_known_file_with_another_size_is_hashed() {
    let scan = Scan::new();
    write(&scan.root.join("a.pdf"), "alpha, longer");

    let files = scan.files(&catalog_with("a.pdf", "alpha"));

    assert_eq!(files[&path("a.pdf")].content_hash, Some(hash_of("alpha, longer")));
}

#[test]
fn a_file_changed_since_the_last_scan_is_hashed_again() {
    let scan = Scan::new();
    let file = write(&scan.root.join("a.pdf"), "alpha");
    let catalog = catalog_with("a.pdf", "alpha");
    scan.files(&catalog);

    write(&file, "ALPHA");
    set_modified(&file, 1_893_456_000);
    let files = scan.files(&catalog);

    assert_eq!(files[&path("a.pdf")].content_hash, Some(hash_of("ALPHA")));
}

#[test]
fn a_cloud_placeholder_is_listed_under_its_real_name_unless_the_file_is_downloaded() {
    let scan = Scan::new();
    write(&scan.root.join(".remote.pdf.icloud"), "stub");
    write(&scan.root.join(".local.pdf.icloud"), "stub");
    write(&scan.root.join("local.pdf"), "beta");

    let files = scan.files(&catalog_with("remote.pdf", "alpha"));

    assert_eq!(files[&path("remote.pdf")], FileFacts { size: 5, content_hash: None });
    assert_eq!(files[&path("local.pdf")].content_hash, Some(hash_of("beta")));
    assert_eq!(files.len(), 2);
}

#[test]
fn a_decomposed_file_name_is_listed_with_its_composed_path() {
    let scan = Scan::new();
    write(&scan.root.join("cafe\u{301}/menu.pdf"), "alpha");

    let files = scan.files(&empty());

    assert_eq!(files.keys().cloned().collect::<Vec<_>>(), [path("caf\u{e9}/menu.pdf")]);
}

#[test]
fn the_listener_learns_the_total_first_and_then_every_file() {
    let scan = Scan::new();
    write(&scan.root.join("a.pdf"), "alpha");
    write(&scan.root.join("letters/b.pdf"), "beta");
    let (first, second) = (RecordingListener::default(), RecordingListener::default());

    scan.scanner.scan(&folder(&scan.root), &empty(), &first).expect("scan");
    scan.scanner.scan(&folder(&scan.root), &empty(), &second).expect("scan");

    assert_eq!(*first.totals.lock().expect("lock"), [2]);
    assert_eq!(first.files(), ["1 a.pdf true", "2 letters/b.pdf true"]);
    assert_eq!(second.files(), ["1 a.pdf false", "2 letters/b.pdf false"]);
}

#[test]
fn a_cancelled_scan_keeps_the_hashes_it_already_computed() {
    let scan = Scan::new();
    write(&scan.root.join("a.pdf"), "alpha");
    write(&scan.root.join("b.pdf"), "beta");
    let checkpointing = FileSystemLibraryScanner::with_checkpoint(scan.temp.path().join("cache"), 1);
    let cancelling = RecordingListener { cancel_after: Some(1), ..RecordingListener::default() };
    let after = RecordingListener::default();

    let cancelled = checkpointing.scan(&folder(&scan.root), &empty(), &cancelling);
    checkpointing.scan(&folder(&scan.root), &empty(), &after).expect("scan");

    assert_eq!(failure_of(cancelled), Some(StorageFailure::Cancelled));
    assert_eq!(after.files(), ["1 a.pdf false", "2 b.pdf true"]);
}

#[test]
fn a_folder_that_is_not_reachable_is_reported_as_such() {
    let scan = Scan::new();
    let offline = folder(&scan.temp.path().join("offline"));

    assert_eq!(
        failure_of(scan.scanner.scan(&offline, &empty(), &NoScanListener)),
        Some(StorageFailure::FolderUnreachable)
    );
}

#[test]
fn an_unreadable_cache_only_costs_hashing_time() {
    let scan = Scan::new();
    write(&scan.root.join("a.pdf"), "alpha");
    write(&scan.temp.path().join("cache/primary.yaml"), "entries: [");

    let files = scan.files(&empty());

    assert_eq!(files[&path("a.pdf")].content_hash, Some(hash_of("alpha")));
}

#[test]
fn the_scan_cache_is_written_as_jackson_wrote_it() {
    let scan = Scan::new();
    for name in ["a.pdf", "2024/b.pdf", "yes.pdf", "Grüße.pdf", "Re: b.pdf"] {
        let file = write(&scan.root.join(name), &format!("content of {name}"));
        set_modified(&file, 1_714_550_400);
    }

    scan.files(&empty());

    assert_eq!(read(&scan.temp.path().join("cache/primary.yaml")), fixture("jackson/scan-cache.yaml"));
}
