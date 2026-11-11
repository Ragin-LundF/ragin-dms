use dms_application::port::state::TaskStore;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory, SuggestedTag};
use dms_domain::document::DocumentId;
use dms_domain::task::TaskKey;
use dms_domain::taxonomy::{CategoryId, TagName};
use dms_infra_storage::FileTaskStore;
use indexmap::IndexSet;

use crate::support::{fixture, read};

fn suggestion(document: &str, model: &str) -> ClassificationSuggestion {
    ClassificationSuggestion::empty(DocumentId::new(document).expect("id"), model)
}

fn full() -> ClassificationSuggestion {
    let mut full = suggestion("doc-1", "test-model");
    full.title = Some("Sample: invoice".to_owned());
    full.category = Some(SuggestedCategory::New {
        name: "Rechnungen".to_owned(),
        parent_id: Some(CategoryId::new("cat-1").expect("id")),
    });
    full.tags = vec![
        SuggestedTag { name: TagName::of("Tax").expect("tag"), is_new: false },
        SuggestedTag { name: TagName::of("2024").expect("tag"), is_new: true },
    ];
    full.document_date = dms_domain::parse_iso_date("2023-12-24");
    full.correspondent = Some("Example Supplier".to_owned());
    full.file_name = Some("2023-12-24 Sample invoice.pdf".to_owned());
    full.transcription = Some("Line one\nLine two\n".to_owned());
    full.confidence = Some(0.75);
    full
}

#[test]
fn suggestions_survive_a_restart_with_every_field_in_the_format_jackson_wrote() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("tasks.yaml");
    let mut existing = suggestion("doc-2", "rules");
    existing.category = Some(SuggestedCategory::Existing(CategoryId::new("cat-1").expect("id")));
    existing.confidence = Some(1.0e-4);
    let mut plain = suggestion("doc-3", "m");
    plain.confidence = Some(1.0);
    let store = FileTaskStore::new(&path);

    for suggestion in [full(), existing.clone(), plain.clone()] {
        store.save_suggestion(&suggestion).expect("saved");
    }
    store.dismiss(&TaskKey::new("new:2024/a.pdf")).expect("dismissed");
    store.dismiss(&TaskKey::new("missing:doc-1")).expect("dismissed");

    assert_eq!(FileTaskStore::new(&path).suggestions().expect("suggestions"), [full(), existing, plain]);
    assert_eq!(read(&path), fixture("jackson/tasks.yaml"));
}

#[test]
fn a_new_suggestion_replaces_the_previous_one_of_the_same_document() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("tasks.yaml");
    let store = FileTaskStore::new(&path);

    store.save_suggestion(&suggestion("doc-1", "a")).expect("saved");
    store.save_suggestion(&suggestion("doc-2", "a")).expect("saved");
    store.save_suggestion(&suggestion("doc-1", "b")).expect("saved");
    store.remove_suggestion(&DocumentId::new("doc-2").expect("id")).expect("removed");

    assert_eq!(FileTaskStore::new(&path).suggestions().expect("suggestions"), [suggestion("doc-1", "b")]);
}

#[test]
fn dismissed_tasks_stay_dismissed_after_a_restart() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("tasks.yaml");
    let store = FileTaskStore::new(&path);

    store.dismiss(&TaskKey::new("new-file:a.pdf")).expect("dismissed");
    store.dismiss(&TaskKey::new("new-file:a.pdf")).expect("dismissed");
    store.dismiss(&TaskKey::new("missing:b.pdf")).expect("dismissed");

    assert_eq!(
        FileTaskStore::new(&path).dismissed_keys().expect("keys"),
        IndexSet::from([TaskKey::new("missing:b.pdf"), TaskKey::new("new-file:a.pdf")])
    );
}

#[test]
fn an_empty_store_writes_both_lists_as_jackson_did() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("tasks.yaml");

    FileTaskStore::new(&path).remove_suggestion(&DocumentId::new("x").expect("id")).expect("written");

    assert_eq!(read(&path), fixture("jackson/tasks-empty.yaml"));
}
