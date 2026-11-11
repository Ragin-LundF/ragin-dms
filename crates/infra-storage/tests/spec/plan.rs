use dms_application::port::plan::{PlanStore, RestructureRecord};
use dms_application::port::storage::StorageFailure;
use dms_domain::document::DocumentId;
use dms_domain::restructure::PlannedMove;
use dms_infra_storage::FilePlanStore;

use crate::catalog::instant;
use crate::support::{failure_of, fixture, folder, path, read, write};

fn record(name: &str) -> RestructureRecord {
    RestructureRecord {
        name: name.to_owned(),
        applied_at: instant("2026-10-22T08:00:00Z"),
        template: "{category}/{year}".to_owned(),
        moves: vec![PlannedMove {
            document_id: DocumentId::new("doc-1").expect("id"),
            from: path("loose/Grüße.pdf"),
            to: path("Letters/2024/Grüße.pdf"),
        }],
    }
}

#[test]
fn a_record_is_written_to_the_hidden_plans_folder_and_read_back() {
    let temp = tempfile::tempdir().expect("temp dir");

    FilePlanStore.write(&folder(temp.path()), &record("20261022-080000")).expect("written");

    assert_eq!(FilePlanStore.read(&folder(temp.path()), "20261022-080000"), Some(record("20261022-080000")));
    assert!(temp.path().join(".dms-plans/20261022-080000.yaml").is_file());
}

#[test]
fn names_are_listed_newest_first_and_a_missing_or_broken_record_reads_as_none() {
    let temp = tempfile::tempdir().expect("temp dir");
    let library = folder(temp.path());
    FilePlanStore.write(&library, &record("20260101-000000")).expect("written");
    FilePlanStore.write(&library, &record("20261022-080000")).expect("written");
    write(&temp.path().join(".dms-plans/broken.yaml"), "moves: [");

    assert_eq!(FilePlanStore.names(&library), ["broken", "20261022-080000", "20260101-000000"]);
    assert_eq!(FilePlanStore.read(&library, "broken"), None);
    assert_eq!(FilePlanStore.read(&library, "missing"), None);
    assert_eq!(FilePlanStore.read(&library, "../escape"), None);
}

#[test]
fn an_offline_folder_fails_to_write_and_has_no_records() {
    let temp = tempfile::tempdir().expect("temp dir");
    let offline = folder(&temp.path().join("offline"));

    assert_eq!(failure_of(FilePlanStore.write(&offline, &record("x"))), Some(StorageFailure::FolderUnreachable));
    assert_eq!(FilePlanStore.names(&offline), Vec::<String>::new());
}

#[test]
fn a_record_is_written_as_jackson_wrote_it() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut record = record("20261022-080000");
    record.moves.push(PlannedMove {
        document_id: DocumentId::new("doc-2").expect("id"),
        from: path("2024.pdf"),
        to: path("yes/2024.pdf"),
    });

    FilePlanStore.write(&folder(temp.path()), &record).expect("written");

    assert_eq!(read(&temp.path().join(".dms-plans/20261022-080000.yaml")), fixture("jackson/plan.yaml"));
}
