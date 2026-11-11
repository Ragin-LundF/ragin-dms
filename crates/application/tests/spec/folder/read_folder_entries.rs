use dms_application::folder::FolderEntriesOutcome;
use dms_application::port::storage::FolderEntries;
use dms_domain::library::RelativePath;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::path;

#[track_caller]
fn read(outcome: FolderEntriesOutcome) -> FolderEntries {
    match outcome {
        FolderEntriesOutcome::Read(entries) => entries,
        other => panic!("expected Read, got {other:?}"),
    }
}

#[test]
fn empty_folders_uncatalogued_files_and_files_only_in_a_sync_folder_are_all_found() {
    let library = TestLibrary::new();
    library.open();
    library.add("home/energy/power.pdf", "power");
    library.create_folder().run(&path("home/empty"));
    library.folders.put(&library.primary, "home/energy/new-scan.pdf", "scan");
    library.folders.put(&library.sync_a, "home/energy/only-here.pdf", "sync");

    let entries = read(library.read_folder_entries().run(&path("home")));

    assert_eq!(
        entries,
        FolderEntries {
            folders: IndexSet::from([path("home/energy"), path("home/empty")]),
            files: IndexSet::from([
                path("home/energy/power.pdf"),
                path("home/energy/new-scan.pdf"),
                path("home/energy/only-here.pdf"),
            ]),
        }
    );
}

#[test]
fn an_offline_sync_folder_adds_nothing_and_a_missing_folder_is_empty() {
    let library = TestLibrary::new();
    library.open();
    library.add("home/power.pdf", "power");
    library.folders.make_unreachable(&library.sync_b);

    let home = read(library.read_folder_entries().run(&path("home")));
    let missing = read(library.read_folder_entries().run(&path("x")));

    assert_eq!(home.files, IndexSet::from([path("home/power.pdf")]));
    assert_eq!(missing, FolderEntries::default());
}

#[test]
fn nothing_is_read_while_no_library_is_open() {
    assert_eq!(
        TestLibrary::new().read_folder_entries().run(&RelativePath::root()),
        FolderEntriesOutcome::LibraryNotOpen
    );
}
