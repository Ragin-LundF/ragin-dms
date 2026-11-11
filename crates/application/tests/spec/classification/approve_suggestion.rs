use dms_application::classification::{ApprovedCategory, SuggestOutcome, SuggestionApproval};
use dms_application::common::OperationOutcome;
use dms_application::port::ai::ClassificationResult;
use dms_application::port::state::TaskStore;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{ClassificationSource, Document};
use dms_domain::field::{CurrencyCode, Decimal, FieldKey, FieldPresets, FieldValue};
use dms_domain::settings::CatalogSettings;
use indexmap::{IndexMap, IndexSet};

use crate::support::TestLibrary;
use crate::support::fake::ai::FakeClassifier;
use crate::support::test_data::{date, done, tag};

fn approval() -> SuggestionApproval {
    SuggestionApproval {
        title: Some("Synthetic statement".to_owned()),
        document_date: Some(date(2024, 2, 15)),
        category: Some(path_of(&["Statements"])),
        tags: IndexSet::from([tag("Tax")]),
        file_name: Some("2024-02 statement.pdf".to_owned()),
        ..SuggestionApproval::default()
    }
}

fn path_of(names: &[&str]) -> ApprovedCategory {
    ApprovedCategory::path(names.iter().map(|name| (*name).to_owned()).collect()).expect("valid path")
}

fn category_only(names: &[&str]) -> SuggestionApproval {
    SuggestionApproval { category: Some(path_of(names)), ..SuggestionApproval::default() }
}

fn suggest_for(library: &TestLibrary, document: &Document) {
    library
        .tasks
        .save_suggestion(&ClassificationSuggestion::empty(document.id.clone(), FakeClassifier::MODEL))
        .expect("saved");
}

#[test]
fn approved_fields_are_written_the_file_is_renamed_everywhere_and_the_suggestion_is_done() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/scan-001.pdf", "scan");
    suggest_for(&library, &document);

    let catalog = done(library.approve_suggestion().run(&document.id, &approval()));

    let approved = &catalog.documents()[&document.id];
    assert_eq!(approved.path.value(), "inbox/2024-02 statement.pdf");
    assert_eq!(approved.metadata.title.as_deref(), Some("Synthetic statement"));
    assert_eq!(approved.metadata.document_date, Some(date(2024, 2, 15)));
    let category = approved.category_id.as_ref().and_then(|id| catalog.categories().get(id));
    assert_eq!(category.map(|category| category.name.as_str()), Some("Statements"));
    assert_eq!(approved.tags, IndexSet::from([tag("tax")]));
    let classification = approved.classification.as_ref().expect("classified");
    assert_eq!(classification.source, ClassificationSource::Ai);
    assert_eq!(classification.model.as_deref(), Some(FakeClassifier::MODEL));
    for folder in library.all_folders() {
        assert_eq!(library.folders.paths(&folder), ["inbox/2024-02 statement.pdf".to_owned()].into());
    }
    assert_eq!(library.tasks.suggestions().expect("readable"), Vec::new());
}

#[test]
fn a_taken_file_name_refuses_the_approval_and_changes_nothing() {
    let library = TestLibrary::new();
    library.open();
    library.add("inbox/2024-02 statement.pdf", "other");
    let document = library.add("inbox/scan-001.pdf", "scan");
    suggest_for(&library, &document);
    let before = library.catalog();

    let outcome = library.approve_suggestion().run(&document.id, &approval());

    assert_eq!(outcome, OperationOutcome::NameTaken);
    assert_eq!(library.catalog(), before);
    assert_eq!(library.tasks.suggestions().expect("readable").len(), 1);
}

#[test]
fn a_new_category_with_the_name_of_an_existing_one_uses_the_existing_one() {
    let library = TestLibrary::new();
    library.open();
    let first = library.add("inbox/a.pdf", "a");
    let second = library.add("inbox/b.pdf", "b");
    suggest_for(&library, &first);
    suggest_for(&library, &second);

    let _ = library.approve_suggestion().run(&first.id, &category_only(&["Statements"]));
    let catalog = done(library.approve_suggestion().run(&second.id, &category_only(&["statements"])));

    assert_eq!(catalog.categories().len(), 1);
    assert_eq!(catalog.documents()[&first.id].category_id, catalog.documents()[&second.id].category_id);
}

#[test]
fn a_path_creates_every_missing_level_and_assigns_the_deepest_one() {
    let library = TestLibrary::new();
    library.open();
    let document = library.add("inbox/a.pdf", "a");
    suggest_for(&library, &document);

    let catalog = done(
        library.approve_suggestion().run(&document.id, &category_only(&["Insurance", "Household", "Rent and living"])),
    );

    assert_eq!(catalog.categories().len(), 3);
    let leaf = &catalog.categories()[catalog.documents()[&document.id].category_id.as_ref().expect("assigned")];
    let middle = &catalog.categories()[leaf.parent_id.as_ref().expect("parent")];
    let top = &catalog.categories()[middle.parent_id.as_ref().expect("parent")];
    assert_eq!(
        [top.name.as_str(), middle.name.as_str(), leaf.name.as_str()],
        ["Insurance", "Household", "Rent and living"]
    );
    assert_eq!(top.parent_id, None);
}

#[test]
fn a_path_reuses_the_levels_that_exist_ignoring_case_and_creates_only_the_rest() {
    let library = TestLibrary::new();
    library.open();
    let first = library.add("inbox/a.pdf", "a");
    let second = library.add("inbox/b.pdf", "b");
    suggest_for(&library, &first);
    suggest_for(&library, &second);
    let _ = library.approve_suggestion().run(&first.id, &category_only(&["Insurance", "Car"]));

    let catalog =
        done(library.approve_suggestion().run(&second.id, &category_only(&["insurance", "Household", "Rent"])));

    let named = |name: &str| {
        let found: Vec<_> = catalog.categories().values().filter(|category| category.name == name).collect();
        assert_eq!(found.len(), 1, "exactly one {name}");
        found[0].clone()
    };
    let (insurance, household, rent) = (named("Insurance"), named("Household"), named("Rent"));
    assert_eq!(catalog.categories().len(), 4);
    assert_eq!(household.parent_id, Some(insurance.id));
    assert_eq!(rent.parent_id, Some(household.id));
    assert_eq!(catalog.documents()[&second.id].category_id, Some(rent.id));
}

#[test]
fn approved_field_values_join_the_values_the_document_has() {
    let library = TestLibrary::new();
    library.open();
    done(
        library.change_settings().run("fields", |_| CatalogSettings {
            field_definitions: FieldPresets::all(),
            ..CatalogSettings::default()
        }),
    );
    let document = library.add("bill.pdf", "bill");
    let amount = FieldValue::Money {
        amount: Decimal::parse("12.50").expect("valid amount"),
        currency: CurrencyCode::new("EUR").expect("valid currency"),
    };
    let due = FieldValue::Date(date(2024, 3, 31));
    let amount_key = FieldPresets::amount().key;
    let due_key = FieldPresets::due_date().key;
    done(
        library
            .store
            .commit(|catalog| catalog.set_fields(&document.id, IndexMap::from([(amount_key.clone(), amount.clone())]))),
    );
    library.classifier.result.set(Some(ClassificationResult::Suggested(ClassificationSuggestion {
        fields: IndexMap::from([
            (due_key.clone(), due.clone()),
            (FieldKey::new("undefined").expect("valid key"), FieldValue::Text("dropped".to_owned())),
        ]),
        ..ClassificationSuggestion::empty(document.id.clone(), FakeClassifier::MODEL)
    })));
    let SuggestOutcome::Suggested(suggestion) = library.suggest_classification().run(&document.id) else {
        panic!("expected Suggested")
    };

    let _ = library
        .approve_suggestion()
        .run(&document.id, &SuggestionApproval { fields: suggestion.fields.clone(), ..SuggestionApproval::default() });

    assert_eq!(suggestion.fields, IndexMap::from([(due_key.clone(), due.clone())]));
    assert_eq!(
        library.catalog().documents()[&document.id].fields,
        IndexMap::from([(amount_key, amount), (due_key, due)])
    );
}
