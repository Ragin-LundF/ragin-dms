use dms_application::catalog::CatalogEdit;
use dms_application::common::OperationOutcome;
use dms_domain::catalog::CatalogRejection;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::{done, tag};

#[test]
fn a_document_is_categorized_and_tagged_by_hand_and_saved_to_every_folder() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");
    let edit = library.edit_catalog();

    edit.run(&CatalogEdit::AddCategory { name: "Statements".to_owned(), parent_id: None });
    let category = library.catalog().categories().values().next().cloned().expect("one category");
    assert_eq!(library.catalog().categories().len(), 1);
    edit.run(&CatalogEdit::AssignCategory { document_id: document.id.clone(), category_id: Some(category.id.clone()) });
    edit.run(&CatalogEdit::AddTags { document_id: document.id.clone(), tags: IndexSet::from([tag("Tax")]) });
    let outcome =
        edit.run(&CatalogEdit::AddTags { document_id: document.id.clone(), tags: IndexSet::from([tag("tax")]) });

    let catalog = done(outcome);
    let edited = &catalog.documents()[&document.id];
    assert_eq!(edited.category_id, Some(category.id));
    assert_eq!(edited.tags.iter().map(dms_domain::taxonomy::TagName::value).collect::<Vec<_>>(), vec!["Tax"]);
    for folder in library.all_folders() {
        assert_eq!(library.folders.catalog_in(&folder).as_ref(), Some(&catalog));
    }
}

#[test]
fn a_category_in_use_cannot_be_removed() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("letters/a.pdf", "alpha");
    let edit = library.edit_catalog();
    edit.run(&CatalogEdit::AddCategory { name: "Statements".to_owned(), parent_id: None });
    let category = library.catalog().categories().values().next().cloned().expect("one category");
    edit.run(&CatalogEdit::AssignCategory { document_id: document.id.clone(), category_id: Some(category.id.clone()) });

    let outcome = edit.run(&CatalogEdit::RemoveCategory { category_id: category.id });

    assert_eq!(outcome, OperationOutcome::Rejected(CatalogRejection::CategoryInUse));
}
