use dms_application::library::OpenLibraryOutcome;
use dms_domain::catalog::Catalog;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::{Applied, library_id};

#[test]
fn a_new_library_starts_with_an_empty_catalog_saved_to_every_folder() {
    let library = TestLibrary::new();

    let catalog = library.open();

    assert!(catalog.documents().is_empty());
    for folder in library.all_folders() {
        assert_eq!(library.folders.catalog_in(&folder).as_ref(), Some(&catalog));
    }
}

#[test]
fn catalog_copies_and_conflict_copies_are_merged_and_saved_to_every_folder() {
    let library = TestLibrary::new();
    let empty = Catalog::empty(library_id("library-1"));
    let first = TestLibrary::document("d1", "letters/a.pdf", "a");
    let second = TestLibrary::document("d2", "letters/b.pdf", "b");
    let third = TestLibrary::document("d3", "letters/c.pdf", "c");
    library.folders.set_catalog(&library.primary, empty.add_document(first.clone()).applied());
    library.folders.set_catalog(&library.sync_a, empty.add_document(second.clone()).applied());
    library.folders.add_conflict_copy(&library.primary, empty.add_document(third.clone()).applied());

    let catalog = library.open();

    let ids: IndexSet<_> = catalog.documents().keys().cloned().collect();
    assert_eq!(ids, IndexSet::from([first.id, second.id, third.id]));
    for folder in library.all_folders() {
        assert_eq!(library.folders.catalog_in(&folder).as_ref(), Some(&catalog));
    }
    assert_eq!(library.folders.conflict_copies_in(&library.primary), Vec::<Catalog>::new());
}

#[test]
fn a_folder_holding_another_library_is_reported_and_not_overwritten() {
    let library = TestLibrary::new();
    let foreign = Catalog::empty(library_id("other-library"));
    library.folders.set_catalog(&library.primary, Catalog::empty(library_id("library-1")));
    library.folders.set_catalog(&library.sync_b, foreign.clone());

    let outcome = library.open_library().run();

    let OpenLibraryOutcome::Opened { catalog, foreign_folders, .. } = outcome else {
        panic!("expected Opened, got {outcome:?}")
    };
    assert_eq!(foreign_folders, IndexSet::from([library.sync_b.id.clone()]));
    assert_eq!(library.folders.catalog_in(&library.sync_b).as_ref(), Some(&foreign));
    assert_eq!(catalog.library_id(), &library_id("library-1"));

    library.add("letters/a.pdf", "alpha");

    assert_eq!(library.folders.catalog_in(&library.sync_b), Some(foreign));
    assert!(library.folders.paths(&library.sync_b).is_empty());
}

#[test]
fn an_unreachable_sync_folder_does_not_keep_the_library_from_opening() {
    let library = TestLibrary::new();
    library.folders.make_unreachable(&library.sync_a);

    let outcome = library.open_library().run();

    let OpenLibraryOutcome::Opened { unreachable_folders, .. } = outcome else {
        panic!("expected Opened, got {outcome:?}")
    };
    assert_eq!(unreachable_folders, IndexSet::from([library.sync_a.id.clone()]));
}

#[test]
fn an_unreachable_primary_folder_opens_nothing() {
    let library = TestLibrary::new();
    library.folders.make_unreachable(&library.primary);

    assert!(matches!(library.open_library().run(), OpenLibraryOutcome::PrimaryFailed(_)));
    assert_eq!(library.store.catalog(), None);
}
