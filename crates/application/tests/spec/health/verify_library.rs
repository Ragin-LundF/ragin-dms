use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::common::{NoProgress, Progress};
use dms_application::health::{VerifyOutcome, VerifyProgress};
use dms_application::port::health::HealthReport;
use dms_domain::health::HealthFinding;
use dms_domain::task::OpenTask;

use crate::support::TestLibrary;
use crate::support::test_data::path;

fn finished(library: &TestLibrary) -> HealthReport {
    match library.verify_library().run(&NoProgress) {
        VerifyOutcome::Finished(report) => report,
        other => panic!("expected Finished, got {other:?}"),
    }
}

/// Cancels after `limit` documents were reported.
struct CancelAfter {
    limit: usize,
    seen: AtomicUsize,
}

impl Progress<VerifyProgress> for CancelAfter {
    fn report(&self, _progress: &VerifyProgress) {
        self.seen.fetch_add(1, Ordering::SeqCst);
    }

    fn is_cancelled(&self) -> bool {
        self.seen.load(Ordering::SeqCst) >= self.limit
    }
}

#[test]
fn an_altered_copy_of_the_same_size_is_damaged_and_repaired_from_an_intact_folder() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.txt", "original");
    library.folders.put(&library.sync_a, "letters/a.txt", "origiNAL");

    let report = finished(&library);

    assert_eq!(
        report.findings,
        vec![HealthFinding::Damaged {
            document_id: document.id.clone(),
            folder_id: library.sync_a.id.clone(),
            source_folder_id: library.primary.id.clone(),
        }]
    );
    assert_eq!(report.resume_after, None);
    assert!(report.finished_at.is_some());
    let checked: Vec<u32> = report.folders.iter().map(|folder| folder.checked).collect();
    assert_eq!(checked, vec![1, 1, 1]);
}

#[test]
fn the_dashboard_shows_the_findings_of_the_last_check() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("a.txt", "original");
    library.folders.put(&library.sync_b, "a.txt", "originaX");
    finished(&library);

    let dashboard = match library.build_dashboard().run(None) {
        dms_application::library::DashboardOutcome::Built(dashboard) => dashboard,
        dms_application::library::DashboardOutcome::LibraryNotOpen => panic!("expected Built"),
    };

    assert!(dashboard.tasks.contains(&OpenTask::DamagedCopy {
        document_id: document.id,
        folder_id: library.sync_b.id.clone(),
        source_folder_id: library.primary.id.clone(),
    }));
}

#[test]
fn not_downloaded_files_are_skipped_and_counted_and_missing_ones_are_no_finding() {
    let library = TestLibrary::new();
    library.open();
    library.add("a.txt", "one");
    library.folders.state.update(|state| state.placeholders.insert((library.sync_a.id.clone(), path("a.txt"))));
    library
        .folders
        .state
        .update(|state| state.files.get_mut(&library.sync_b.id).map(|files| files.shift_remove(&path("a.txt"))));

    let report = finished(&library);

    assert_eq!(report.findings, Vec::<HealthFinding>::new());
    let sync_a = report.folders.iter().find(|folder| folder.folder_id == library.sync_a.id).expect("sync a");
    let sync_b = report.folders.iter().find(|folder| folder.folder_id == library.sync_b.id).expect("sync b");
    assert_eq!((sync_a.checked, sync_a.not_downloaded), (0, 1));
    assert_eq!((sync_b.checked, sync_b.missing), (0, 1));
}

#[test]
fn a_pdf_that_opens_nowhere_is_unreadable() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("scan.pdf", "BROKEN pdf");

    let report = finished(&library);

    assert_eq!(report.findings, vec![HealthFinding::Unreadable { document_id: document.id }]);
}

#[test]
fn an_offline_folder_keeps_its_findings_and_counts_as_offline() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("a.txt", "original");
    library.folders.put(&library.sync_a, "a.txt", "originaX");
    finished(&library);
    library.folders.make_unreachable(&library.sync_a);

    let report = finished(&library);

    let sync_a = report.folders.iter().find(|folder| folder.folder_id == library.sync_a.id).expect("sync a");
    assert!(sync_a.offline);
    assert_eq!(
        report.findings,
        vec![HealthFinding::Damaged {
            document_id: document.id,
            folder_id: library.sync_a.id.clone(),
            source_folder_id: library.primary.id.clone(),
        }]
    );
}

#[test]
fn a_cancelled_check_resumes_after_the_last_checked_document() {
    let library = TestLibrary::new();
    library.open();
    library.add("a.txt", "one");
    library.add("b.txt", "two");
    library.add("c.txt", "three");

    let cancelled = library.verify_library().run(&CancelAfter { limit: 1, seen: AtomicUsize::new(0) });
    let VerifyOutcome::Cancelled(partial) = cancelled else { panic!("expected Cancelled, got {cancelled:?}") };
    assert!(partial.resume_after.is_some());
    assert_eq!(partial.finished_at, None);

    let counter = CancelAfter { limit: usize::MAX, seen: AtomicUsize::new(0) };
    let VerifyOutcome::Finished(report) = library.verify_library().run(&counter) else { panic!("expected Finished") };

    assert_eq!(counter.seen.load(Ordering::SeqCst), 2, "only the two unchecked documents are read again");
    assert_eq!(report.folders.iter().map(|folder| folder.checked).collect::<Vec<_>>(), vec![3, 3, 3]);
}

#[test]
fn a_finding_whose_copy_was_fixed_disappears_at_the_next_check() {
    let library = TestLibrary::new();
    library.open();
    library.add("a.txt", "original");
    library.folders.put(&library.sync_a, "a.txt", "originaX");
    finished(&library);
    library.folders.put(&library.sync_a, "a.txt", "original");

    assert_eq!(finished(&library).findings, Vec::<HealthFinding>::new());
}

#[test]
fn nothing_is_checked_while_no_library_is_open() {
    let library = TestLibrary::new();
    assert_eq!(library.verify_library().run(&NoProgress), VerifyOutcome::LibraryNotOpen);
}
