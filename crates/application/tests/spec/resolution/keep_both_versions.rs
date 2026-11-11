use dms_application::common::OperationOutcome;

use crate::support::TestLibrary;
use crate::support::test_data::path;

#[test]
fn one_version_is_renamed_and_copied_into_every_folder() {
    let library = TestLibrary::new();
    library.open();
    library.folders.put(&library.primary, "notes/n.txt", "one");
    library.folders.put(&library.sync_a, "notes/n.txt", "two");

    let outcome = library.keep_both_versions().run(&path("notes/n.txt"), &library.sync_a.id);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "notes/n (2).txt").as_deref(), Some("two"));
    }
    assert_eq!(library.folders.content(&library.primary, "notes/n.txt").as_deref(), Some("one"));
}
