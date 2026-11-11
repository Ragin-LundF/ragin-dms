use std::collections::{BTreeSet, HashSet};

use dms_application::common::OperationOutcome;
use dms_application::resolution::DuplicatePreference;
use dms_domain::document::{Document, DocumentMetadata};
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::{path, tag};

#[test]
fn keeping_one_copy_trashes_the_other_everywhere_and_keeps_what_was_assigned_to_it() {
    let library = TestLibrary::new();
    library.open();
    let kept = library.add("letters/a.pdf", "alpha");
    let copy = library.add("letters/b.pdf", "alpha");
    library.store.commit(|catalog| {
        let tagged = catalog.set_tags(&copy.id, &IndexSet::from([tag("energy")]));
        let dms_domain::catalog::CatalogUpdate::Applied(tagged) = tagged else { return Ok(tagged) };
        tagged.update_metadata(
            &copy.id,
            DocumentMetadata { title: Some("Bill".to_owned()), ..DocumentMetadata::default() },
        )
    });

    let refused = library.merge_duplicates().run(&kept.id, false);
    let outcome = library.merge_duplicates().run(&kept.id, true);

    assert_eq!(refused, OperationOutcome::NotConfirmed);
    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    let catalog = library.catalog();
    assert_eq!(catalog.documents().len(), 1);
    let document = catalog.documents().values().next().expect("one document");
    assert_eq!(document.id, kept.id);
    let tags: BTreeSet<&str> = document.tags.iter().map(dms_domain::taxonomy::TagName::value).collect();
    assert_eq!(tags, BTreeSet::from(["energy"]));
    assert_eq!(document.metadata.title.as_deref(), Some("Bill"));
    let trashed: HashSet<_> =
        library.all_folders().into_iter().map(|folder| (folder.id, path("letters/b.pdf"))).collect();
    assert_eq!(library.folders.trashed().into_iter().collect::<HashSet<_>>(), trashed);
}

#[test]
fn nothing_changes_when_the_copy_in_the_primary_differs_byte_for_byte() {
    let library = TestLibrary::new();
    library.open();
    let kept = library.add("letters/a.pdf", "alpha");
    library.add("letters/b.pdf", "alpha");
    library.folders.put(&library.primary, "letters/b.pdf", "alphb");

    let outcome = library.merge_duplicates().run(&kept.id, true);

    assert_eq!(outcome, OperationOutcome::ScanOutdated);
    assert_eq!(library.folders.trashed(), Vec::new());
    assert_eq!(library.catalog().documents().len(), 2);
}

#[test]
fn a_sync_folder_whose_copy_differs_keeps_its_file() {
    let library = TestLibrary::new();
    library.open();
    let kept = library.add("letters/a.pdf", "alpha");
    library.add("letters/b.pdf", "alpha");
    library.folders.put(&library.sync_a, "letters/b.pdf", "alphb");

    let outcome = library.merge_duplicates().run(&kept.id, true);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    assert_eq!(library.folders.content(&library.sync_a, "letters/b.pdf").as_deref(), Some("alphb"));
    let trashed: HashSet<_> = [&library.primary, &library.sync_b]
        .into_iter()
        .map(|folder| (folder.id.clone(), path("letters/b.pdf")))
        .collect();
    assert_eq!(library.folders.trashed().into_iter().collect::<HashSet<_>>(), trashed);
}

#[test]
fn the_copy_with_the_most_assigned_values_is_kept_by_default() {
    let library = TestLibrary::new();
    library.open();
    let plain = library.add("letters/a.pdf", "alpha");
    let tagged = library.add("letters/b.pdf", "alpha");
    library.store.commit(|catalog| Ok(catalog.set_tags(&tagged.id, &IndexSet::from([tag("energy")]))));
    let documents: Vec<Document> = library
        .catalog()
        .documents()
        .values()
        .filter(|document| document.id == plain.id || document.id == tagged.id)
        .cloned()
        .collect();

    assert_eq!(DuplicatePreference::preferred(&documents).map(|document| &document.id), Some(&tagged.id));
}
