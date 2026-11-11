use dms_domain::change::{ContentConflict, DetectedChange, NewFile};
use dms_domain::library::LibraryFolderId;
use dms_domain::task::{OpenTask, TaskKey};
use indexmap::{IndexMap, indexset};

use crate::test_data::{hash, path};

fn new_file() -> NewFile {
    NewFile {
        path: path("inbox/scan.pdf"),
        present_in: indexset! { LibraryFolderId::new("main").expect("valid") },
        size: 1,
        content_hash: Some(hash(1)),
    }
}

#[test]
fn every_detected_change_maps_to_the_task_that_resolves_it() {
    assert!(matches!(OpenTask::for_change(DetectedChange::NewFile(new_file())), OpenTask::AdoptNewFile(_)));
    let conflict = ContentConflict { path: path("inbox/scan.pdf"), content_hashes: IndexMap::new() };
    assert!(matches!(
        OpenTask::for_change(DetectedChange::ContentConflict(conflict)),
        OpenTask::ResolveContentConflict(_)
    ));
}

#[test]
fn the_same_change_found_twice_yields_the_same_key() {
    let first = OpenTask::for_change(DetectedChange::NewFile(new_file()));
    let second = OpenTask::for_change(DetectedChange::NewFile(NewFile { size: 2, ..new_file() }));

    assert_eq!(first.key(), TaskKey::new("new:inbox/scan.pdf"));
    assert_eq!(first.key(), second.key());
}
