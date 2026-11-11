use dms_application::classification::{SuggestOutcome, SuggestionApproval};
use dms_application::port::ai::ClassificationResult;
use dms_domain::classification::{ClassificationSuggestion, SuggestedFolder};
use dms_domain::document::Document;
use dms_domain::settings::InboxSettings;

use crate::support::TestLibrary;
use crate::support::fake::ai::FakeClassifier;
use crate::support::test_data::{done, path};

fn library_with_inbox_document(ai_target_folder: bool) -> (TestLibrary, Document) {
    let library = TestLibrary::new();
    library.open();
    library.add("Home/Power/old bill.pdf", "old");
    done(library.configure_inbox().run(&InboxSettings { enabled: true, ai_target_folder, ..InboxSettings::default() }));
    let document = library.add("Inbox/scan.pdf", "new");
    (library, document)
}

fn suggest_folder(library: &TestLibrary, document: &Document, folder: &str, is_new: bool) {
    library.classifier.result.set(Some(ClassificationResult::Suggested(ClassificationSuggestion {
        target_folder: Some(SuggestedFolder { path: path(folder), is_new }),
        target_reason: Some("Other power bills are there".to_owned()),
        ..ClassificationSuggestion::empty(document.id.clone(), FakeClassifier::MODEL)
    })));
}

fn suggested(library: &TestLibrary, document: &Document) -> ClassificationSuggestion {
    match library.suggest_classification().run(&document.id) {
        SuggestOutcome::Suggested(suggestion) => suggestion,
        other => panic!("expected Suggested, got {other:?}"),
    }
}

#[test]
fn an_inbox_document_gets_the_folder_outline_without_the_inbox() {
    let (library, document) = library_with_inbox_document(true);

    let _ = library.suggest_classification().run(&document.id);

    let requests = library.classifier.requests.get();
    let outline = requests[0].outline.as_ref().expect("an outline");
    assert_eq!(outline.folders.iter().map(|it| it.path.value()).collect::<Vec<_>>(), ["Home", "Home/Power"]);
}

#[test]
fn without_the_target_folder_setting_there_is_no_outline() {
    let (library, document) = library_with_inbox_document(false);

    let _ = library.suggest_classification().run(&document.id);

    assert!(library.classifier.requests.get()[0].outline.is_none());
}

#[test]
fn an_existing_and_a_new_target_folder_are_kept_one_outside_the_outline_is_dropped() {
    let (library, document) = library_with_inbox_document(true);

    suggest_folder(&library, &document, "Home/Power", true);
    let existing = suggested(&library, &document);
    suggest_folder(&library, &document, "Home/Water", true);
    let created = suggested(&library, &document);
    suggest_folder(&library, &document, "Away/Water", true);
    let outside = suggested(&library, &document);

    assert_eq!(existing.target_folder, Some(SuggestedFolder { path: path("Home/Power"), is_new: false }));
    assert_eq!(created.target_folder.map(|it| it.is_new), Some(true));
    assert_eq!(outside.target_folder, None);
    assert_eq!(outside.target_reason, None);
}

#[test]
fn approving_files_the_document_into_the_target_folder_with_the_new_name_in_one_move() {
    let (library, document) = library_with_inbox_document(true);
    suggest_folder(&library, &document, "Home/Water", true);
    let _ = library.suggest_classification().run(&document.id);

    let outcome = library.approve_suggestion().run(
        &document.id,
        &SuggestionApproval {
            file_name: Some("Water bill.pdf".to_owned()),
            target_folder: Some(path("Home/Water")),
            ..SuggestionApproval::default()
        },
    );

    done(outcome);
    assert_eq!(library.catalog().documents()[&document.id].path.value(), "Home/Water/Water bill.pdf");
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, "Home/Water/Water bill.pdf").as_deref(), Some("new"));
    }
}
