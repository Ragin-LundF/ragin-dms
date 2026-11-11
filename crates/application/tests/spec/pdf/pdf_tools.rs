use dms_application::catalog::CatalogEdit;
use std::collections::BTreeSet;

use dms_application::common::OperationOutcome;
use dms_application::port::pdf::PageChange;
use dms_application::port::storage::{FolderOperation, OperationJournal};
use dms_domain::document::DocumentMetadata;
use dms_domain::taxonomy::Category;
use indexmap::IndexSet;

use super::kit::PdfKit;
use crate::support::fake::storage::FakeFolders;
use crate::support::test_data::{category_id, done, tag};

#[test]
fn editing_pages_makes_a_new_version_everywhere_and_the_old_one_goes_to_the_trash() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("letters/scan.pdf", "one|two|three");

    let outcome = library
        .edit_pages()
        .run(&scan.id, &[PageChange { page: 2, rotation: 90 }, PageChange { page: 0, rotation: 0 }]);

    done(outcome);
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "letters/scan.pdf").as_deref(), Some("three@90|one"));
        assert!(library.folders.trashed().contains(&(folder.id.clone(), scan.path.clone())));
    }
    let edited = library.catalog().documents()[&scan.id].clone();
    assert_eq!(edited.content_hash, FakeFolders::hash_of("three@90|one"));
    assert_eq!(library.folders.paths(&library.primary), BTreeSet::from(["letters/scan.pdf".to_owned()]));
}

#[test]
fn an_offline_sync_folder_gets_the_new_version_from_the_journal() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("scan.pdf", "one|two");
    library.folders.make_unreachable(&library.sync_b);

    let _ = library.edit_pages().run(&scan.id, &[PageChange { page: 1, rotation: 0 }]);

    let expected = FolderOperation::CopyFromPrimary { path: scan.path.clone(), replace_existing: true };
    assert!(library.journal.pending().expect("journal").iter().any(|pending| pending.operation == expected));
}

#[test]
fn splitting_creates_documents_that_inherit_category_tags_and_correspondent_and_keeps_the_original() {
    let library = PdfKit::new();
    library.open();
    let bills = Category::new(category_id("bills"), "Bills", None, None).expect("valid category");
    done(library.store.commit(|catalog| catalog.add_category(bills.clone())));
    let scan = library.add("letters/batch.pdf", "a|b|c");
    let edits = [
        CatalogEdit::AssignCategory { document_id: scan.id.clone(), category_id: Some(bills.id.clone()) },
        CatalogEdit::AddTags { document_id: scan.id.clone(), tags: IndexSet::from([tag("Tax")]) },
        CatalogEdit::UpdateMetadata {
            document_id: scan.id.clone(),
            metadata: DocumentMetadata {
                correspondent: Some("Sample Energy".to_owned()),
                ..DocumentMetadata::default()
            },
        },
    ];
    for edit in &edits {
        let _ = library.edit_catalog().run(edit);
    }

    let outcome = library.split_document().run(&scan.id, &[0..=0, 1..=2], false);

    done(outcome);
    let catalog = library.catalog();
    let mut parts: Vec<_> = catalog.documents().values().filter(|document| document.id != scan.id).collect();
    parts.sort_by(|first, second| first.path.value().cmp(second.path.value()));
    assert_eq!(
        parts.iter().map(|part| part.path.value()).collect::<Vec<_>>(),
        ["letters/batch (page 1).pdf", "letters/batch (pages 2-3).pdf"]
    );
    for part in &parts {
        assert_eq!(part.category_id.as_ref(), Some(&bills.id));
        assert_eq!(part.tags, IndexSet::from([tag("Tax")]));
        assert_eq!(part.metadata.correspondent.as_deref(), Some("Sample Energy"));
    }
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "letters/batch (pages 2-3).pdf").as_deref(), Some("b|c"));
    }
    assert!(catalog.documents().contains_key(&scan.id));
}

#[test]
fn splitting_with_the_confirmed_choice_moves_the_original_to_the_trash() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("batch.pdf", "a|b");

    let _ = library.split_document().run(&scan.id, &[0..=0, 1..=1], true);

    assert!(!library.catalog().documents().contains_key(&scan.id));
    assert_eq!(library.catalog().documents().len(), 2);
}

#[test]
fn merging_keeps_the_order_and_trashes_the_sources_only_when_confirmed() {
    let library = PdfKit::new();
    library.open();
    let first = library.add("first.pdf", "1");
    let second = library.add("second.pdf", "2");

    let _ = library.merge_documents().run(&[second.id.clone(), first.id.clone()], "both", false);

    assert_eq!(library.folders.content(&library.sync_a, "both.pdf").as_deref(), Some("2|1"));
    assert_eq!(library.catalog().documents().len(), 3);

    let outcome = library.merge_documents().run(&[first.id.clone(), second.id.clone()], "both.pdf", true);

    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
    assert_eq!(library.folders.content(&library.primary, "both (2).pdf").as_deref(), Some("1|2"));
    let paths: BTreeSet<String> =
        library.catalog().documents().values().map(|document| document.path.value().to_owned()).collect();
    assert_eq!(paths, BTreeSet::from(["both.pdf".to_owned(), "both (2).pdf".to_owned()]));
}
