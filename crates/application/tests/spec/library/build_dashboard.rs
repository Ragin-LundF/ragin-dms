use dms_application::library::DashboardOutcome;
use dms_application::port::state::TaskStore;
use dms_domain::task::OpenTask;

use crate::support::TestLibrary;

#[test]
fn scan_results_and_failed_operations_become_tasks() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.sync_b);
    library.add("letters/a.pdf", "alpha");
    library.folders.put(&library.sync_a, "inbox/new.pdf", "new");

    let scan = library.scanned(false).expect("scanned");
    let dashboard = library.dashboard(Some(&scan)).expect("built");

    assert!(dashboard.tasks.iter().any(|task| matches!(task, OpenTask::AdoptNewFile(_))));
    assert!(
        dashboard.tasks.contains(&OpenTask::RetryFailedOperations { folder_id: library.sync_b.id.clone(), count: 2 })
    );
    let sync_b = dashboard.folders.iter().find(|state| state.folder == library.sync_b).expect("sync-b listed");
    assert_eq!(sync_b.reachable, Some(false));
    assert_eq!(dashboard.document_count, 1);
    assert_eq!(dashboard.uncategorized_count, 1);
}

#[test]
fn a_dismissed_task_stays_hidden_when_the_scan_finds_the_change_again() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.sync_a, "inbox/new.pdf", "new");
    let first_scan = library.scanned(false).expect("scanned");
    let tasks = library.dashboard(Some(&first_scan)).expect("built").tasks;
    assert_eq!(tasks.len(), 1);

    // Kotlin's `DismissTask` only stores the key; the resolution tests cover that use case itself.
    library.tasks.dismiss(&tasks[0].key()).expect("dismissed");
    let dashboard = library.dashboard(library.scanned(false).as_ref()).expect("built");

    assert_eq!(dashboard.tasks, Vec::<OpenTask>::new());
}

#[test]
fn no_dashboard_is_built_while_no_library_is_open() {
    assert_eq!(TestLibrary::new().build_dashboard().run(None), DashboardOutcome::LibraryNotOpen);
}
