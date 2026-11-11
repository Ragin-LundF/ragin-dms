use dms_application::catalog::CatalogEdit;
use dms_application::port::system::Clock;
use dms_application::search::{FindOutcome, SavedSearchPresets};
use dms_domain::document::DocumentId;
use indexmap::{IndexMap, IndexSet};

use crate::support::TestLibrary;
use crate::support::test_data::tag;

#[test]
fn the_presets_find_untagged_uncategorized_and_this_years_tax_documents() {
    let library = TestLibrary::new();
    library.open();
    let tax = tag("Tax");
    let plain = library.add("plain.pdf", "a");
    let taxed = library.add("2024-02-01 tax.pdf", "b");
    library
        .edit_catalog()
        .run(&CatalogEdit::AddTags { document_id: taxed.id.clone(), tags: IndexSet::from([tax.clone()]) });

    let presets = SavedSearchPresets::of(library.clock.today(), library.ids.as_ref(), &tax);
    let found: IndexMap<String, Vec<DocumentId>> = presets
        .iter()
        .map(|preset| match library.find_documents().run(&preset.query) {
            FindOutcome::Found(documents) => {
                (preset.name.clone(), documents.into_iter().map(|document| document.id).collect())
            }
            FindOutcome::LibraryNotOpen => panic!("expected Found"),
        })
        .collect();

    let expected: IndexMap<String, Vec<DocumentId>> = IndexMap::from([
        ("Untagged".to_owned(), vec![plain.id.clone()]),
        ("Uncategorized".to_owned(), vec![taxed.id.clone(), plain.id]),
        ("Tax 2024".to_owned(), vec![taxed.id]),
        ("Due in 30 days".to_owned(), Vec::new()),
    ]);
    assert_eq!(found, expected);
}
