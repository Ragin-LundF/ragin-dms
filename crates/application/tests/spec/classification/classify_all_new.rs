use std::sync::Mutex;

use dms_application::classification::{ClassificationProgress, SuggestOutcome};
use dms_domain::library::RelativePath;

use crate::support::TestLibrary;

fn classify_all(library: &TestLibrary) -> Vec<ClassificationProgress> {
    let reports = Mutex::new(Vec::new());
    library.classify_all_new().run(&RelativePath::root(), &|step: &ClassificationProgress| {
        reports.lock().expect("lock").push(step.clone());
    });
    reports.into_inner().expect("lock")
}

#[test]
fn every_readable_document_without_a_classification_gets_a_suggestion_once() {
    let library = TestLibrary::new();
    library.open();
    library.add("inbox/a.pdf", "a");
    library.add("inbox/b.png", "b");
    library.add("inbox/c.txt", "c");

    let progress = classify_all(&library);
    let again = classify_all(&library);

    assert_eq!(progress.iter().map(|it| it.done).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(progress.iter().map(|it| it.total).collect::<Vec<_>>(), [2, 2]);
    assert!(progress.iter().all(|it| matches!(it.outcome, SuggestOutcome::Suggested(_))));
    assert_eq!(again, Vec::new());
}
