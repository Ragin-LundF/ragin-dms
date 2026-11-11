use dms_application::common::OperationOutcome;
use dms_application::inbox::CollectInboxOutcome;
use dms_application::port::storage::{FolderOperation, OperationJournal};
use dms_domain::catalog::CatalogRejection;
use dms_domain::settings::InboxSettings;
use indexmap::IndexSet;

use super::scan_now;
use crate::support::TestLibrary;
use crate::support::test_data::{done, path};

fn inbox() -> InboxSettings {
    InboxSettings { enabled: true, path: path("Inbox"), ..InboxSettings::default() }
}

fn opened_with_inbox() -> TestLibrary {
    let library = TestLibrary::new();
    library.open();
    done(library.configure_inbox().run(&inbox()));
    library
}

#[test]
fn switching_the_inbox_on_creates_its_folder_everywhere_and_stores_the_settings_in_the_catalog() {
    let library = TestLibrary::new();
    library.open();

    let catalog = done(library.configure_inbox().run(&inbox()));

    assert_eq!(catalog.settings().layout.inbox, inbox());
    assert_eq!(library.catalog().settings().layout.inbox, inbox());
    for folder in library.all_folders() {
        assert!(library.folders.has_directory(&folder, "Inbox"));
    }
    let sync_b = library.folders.catalog_in(&library.sync_b).expect("a catalog");
    assert_eq!(sync_b.settings().layout.inbox, inbox());
}

#[test]
fn an_offline_sync_folder_gets_the_inbox_folder_from_the_journal() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.sync_b);

    let outcome = library.configure_inbox().run(&inbox());

    let OperationOutcome::Done { pending_folders, .. } = outcome else { panic!("expected Done, got {outcome:?}") };
    assert_eq!(pending_folders, IndexSet::from([library.sync_b.id.clone()]));
    let pending = library.journal.pending().expect("readable");
    assert!(pending.iter().any(|it| it.operation == FolderOperation::CreateFolder(inbox().path)));
}

#[test]
fn a_file_dropped_into_a_sync_folder_s_inbox_reaches_the_primary_and_every_other_folder() {
    let library = opened_with_inbox();
    library.folders.put(&library.sync_a, "Inbox/scan-7.pdf", "dropped");
    let scan = scan_now(&library);

    let outcome = library.collect_inbox().run(&scan);

    let CollectInboxOutcome::Collected(collected) = outcome else { panic!("expected Collected, got {outcome:?}") };
    assert_eq!(collected.iter().map(|it| it.path.value()).collect::<Vec<_>>(), ["Inbox/scan-7.pdf"]);
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "Inbox/scan-7.pdf").as_deref(), Some("dropped"));
    }
    assert!(library.catalog().find_by_path(&path("Inbox/scan-7.pdf")).is_some());
}

#[test]
fn new_files_outside_the_inbox_are_left_to_the_dashboard() {
    let library = opened_with_inbox();
    library.folders.put(&library.primary, "elsewhere.pdf", "x");
    let scan = scan_now(&library);

    assert_eq!(library.collect_inbox().run(&scan), CollectInboxOutcome::Collected(Vec::new()));
}

#[test]
fn filing_by_hand_moves_the_document_into_a_new_folder_everywhere() {
    let library = opened_with_inbox();
    let document = library.add("Inbox/scan-1.pdf", "a");

    let outcome = library.file_inbox_document().run(&document.id, &path("Home/Power"), Some("Power bill.pdf"));

    done(outcome);
    let catalog = library.catalog();
    let filed = &catalog.documents()[&document.id];
    assert_eq!(filed.path.value(), "Home/Power/Power bill.pdf");
    for folder in library.all_folders() {
        assert_eq!(library.folders.paths(&folder), ["Home/Power/Power bill.pdf".to_owned()].into());
    }
    assert!(!catalog.settings().layout.inbox.contains(&filed.path));
}

#[test]
fn a_name_taken_in_the_target_folder_gets_a_numbered_suffix() {
    let library = opened_with_inbox();
    library.add("Home/scan.pdf", "old");
    let document = library.add("Inbox/SCAN.pdf", "new");

    let _ = library.file_inbox_document().run(&document.id, &path("Home"), None);

    assert_eq!(library.catalog().documents()[&document.id].path.value(), "Home/SCAN (2).pdf");
}

#[test]
fn filing_into_the_inbox_itself_is_refused() {
    let library = opened_with_inbox();
    let document = library.add("Inbox/a.pdf", "a");

    assert_eq!(
        library.file_inbox_document().run(&document.id, &path("Inbox/x"), None),
        OperationOutcome::Rejected(CatalogRejection::InvalidFolderMove)
    );
}
