use std::sync::Mutex;

use dms_application::classification::ClassificationProgress;

use super::add_tags;
use crate::support::TestLibrary;
use crate::support::test_data::tag;

fn tag_all(library: &TestLibrary) -> Vec<ClassificationProgress> {
    let reports = Mutex::new(Vec::new());
    library.tag_all_untagged().run(&|step: &ClassificationProgress| reports.lock().expect("lock").push(step.clone()));
    reports.into_inner().expect("lock")
}

#[test]
fn only_readable_documents_without_tags_are_tagged_each_once() {
    let library = TestLibrary::new();
    library.open();
    library.add("inbox/a.pdf", "a");
    let tagged = library.add("inbox/b.pdf", "b");
    library.add("inbox/c.txt", "c");
    add_tags(&library, &tagged.id, &[tag("Home")]);

    let progress = tag_all(&library);
    let again = tag_all(&library);

    let catalog = library.catalog();
    let paths: Vec<&str> = progress.iter().map(|it| catalog.documents()[&it.document_id].path.value()).collect();
    assert_eq!(paths, ["inbox/a.pdf"]);
    assert_eq!(again, Vec::new());
}
