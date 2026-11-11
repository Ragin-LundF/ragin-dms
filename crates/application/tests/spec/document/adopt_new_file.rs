use dms_application::common::OperationOutcome;
use dms_domain::change::DetectedChange;

use crate::support::TestLibrary;
use crate::support::fake::storage::FakeFolders;
use crate::support::test_data::path;

#[test]
fn a_file_found_in_one_sync_folder_is_catalogued_and_copied_into_every_folder() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.sync_a, "inbox/scan.pdf", "scan");
    let new_file = library
        .scan_changes()
        .into_iter()
        .find_map(|change| match change {
            DetectedChange::NewFile(new_file) => Some(new_file),
            _ => None,
        })
        .expect("a new file");

    let outcome = library.adopt_new_file().run(&new_file);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "inbox/scan.pdf").as_deref(), Some("scan"));
    }
    let catalog = library.catalog();
    let document = catalog.find_by_path(&path("inbox/scan.pdf")).expect("catalogued");
    assert_eq!(document.content_hash, FakeFolders::hash_of("scan"));
}
