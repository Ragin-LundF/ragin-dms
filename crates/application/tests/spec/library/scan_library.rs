use std::sync::Mutex;

use dms_application::common::{NoProgress, Progress};
use dms_application::library::{ScanOutcome, ScanProgress};
use dms_application::port::state::{ActivityKind, ActivityLog};
use dms_domain::change::DetectedChange;

use crate::support::TestLibrary;
use crate::support::test_data::path;

#[test]
fn progress_names_each_folder_and_counts_its_files() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.primary, "a.pdf", "alpha");
    library.folders.put(&library.primary, "b.pdf", "beta");
    let reported = Mutex::new(Vec::<ScanProgress>::new());

    library.scanned_with(false, &|step: &ScanProgress| reported.lock().expect("lock").push(step.clone()));

    let reported = reported.into_inner().expect("lock");
    let primary: Vec<&ScanProgress> = reported.iter().filter(|step| step.folder_number == 1).collect();
    assert_eq!(reported.last().expect("reported").folder_count, 3);
    assert_eq!(primary.iter().map(|step| step.total).collect::<Vec<_>>(), vec![None, Some(2), Some(2), Some(2)]);
    assert_eq!(primary.iter().map(|step| step.done).collect::<Vec<_>>(), vec![0, 0, 1, 2]);
    assert_eq!(primary.last().expect("reported").current, Some(path("b.pdf")));
}

#[test]
fn without_adding_a_scan_changes_nothing() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.primary, "a.pdf", "alpha");

    let scan = library.scanned(false).expect("scanned");

    assert_eq!(library.catalog().documents().len(), 0);
    assert_eq!(scan.changes.iter().filter(|change| matches!(change, DetectedChange::NewFile(_))).count(), 1);
}

#[test]
fn adding_catalogs_new_files_of_the_primary_and_leaves_the_rest_to_the_dashboard() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.primary, "letters/a.pdf", "alpha");
    library.folders.put(&library.primary, "copy-of-a.pdf", "alpha");
    library.folders.put(&library.sync_a, "inbox/scan.pdf", "scan");

    let scan = library.scanned(true).expect("scanned");

    let paths: Vec<String> =
        library.catalog().documents().values().map(|document| document.path.value().to_owned()).collect();
    assert_eq!(paths.len(), 1);
    assert!(["letters/a.pdf", "copy-of-a.pdf"].contains(&paths[0].as_str()));
    let new_files: Vec<&str> = scan
        .changes
        .iter()
        .filter_map(|change| match change {
            DetectedChange::NewFile(new) => Some(new.path.value()),
            _ => None,
        })
        .collect();
    assert!(new_files.contains(&"inbox/scan.pdf"));
    let kinds: Vec<ActivityKind> =
        library.activity_log.recent(10).expect("readable").iter().map(|entry| entry.kind).collect();
    assert_eq!(kinds, vec![ActivityKind::DocumentAdded]);
}

#[test]
fn nothing_is_scanned_while_no_library_is_open() {
    assert_eq!(TestLibrary::new().scan_library().run(false, &NoProgress), ScanOutcome::LibraryNotOpen);
}

/// Cancels the scan as soon as the first file of the primary is read.
struct CancelAfterFirstFile(Mutex<bool>);

impl Progress<ScanProgress> for CancelAfterFirstFile {
    fn report(&self, step: &ScanProgress) {
        if step.done >= 1 {
            *self.0.lock().expect("lock") = true;
        }
    }

    fn is_cancelled(&self) -> bool {
        *self.0.lock().expect("lock")
    }
}

#[test]
fn a_cancelled_scan_still_catalogues_what_it_found_and_reports_the_cancellation() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.primary, "a.pdf", "alpha");

    let outcome = library.scan_library().run(true, &CancelAfterFirstFile(Mutex::new(false)));

    assert_eq!(outcome, ScanOutcome::Cancelled);
    assert!(library.catalog().find_by_path(&path("a.pdf")).is_some());
}
