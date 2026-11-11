use dms_application::folder::FoldersOutcome;
use dms_application::port::storage::StorageFailure;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::path;

#[test]
fn the_folders_of_the_primary_are_read_also_empty_ones_and_a_folder_only_in_a_sync_folder_is_not() {
    let library = TestLibrary::new();
    library.open();
    library.add("home/energy/power.pdf", "power");
    library.create_folder().run(&path("home/empty"));
    library.folders.put(&library.sync_a, "elsewhere/only-here.pdf", "sync");

    let FoldersOutcome::Read(folders) = library.read_folders().run() else { panic!("expected Read") };

    assert_eq!(folders, IndexSet::from([path("home"), path("home/energy"), path("home/empty")]));
}

#[test]
fn an_unreachable_primary_is_a_failure_and_nothing_is_read_while_no_library_is_open() {
    let library = TestLibrary::new();
    library.open();
    library.folders.make_unreachable(&library.primary);

    let FoldersOutcome::StorageFailed(failure) = library.read_folders().run() else { panic!("expected StorageFailed") };

    assert_eq!(failure.reason, StorageFailure::FolderUnreachable);
    assert_eq!(TestLibrary::new().read_folders().run(), FoldersOutcome::LibraryNotOpen);
}
