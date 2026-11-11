use std::fs;

use chrono::Duration;
use dms_application::port::state::{ActivityEntry, ActivityKind, ActivityLog};
use dms_infra_storage::FileActivityLog;

use crate::catalog::instant;
use crate::support::{fixture, read};

fn entry(minute: i64, subject: &str) -> ActivityEntry {
    ActivityEntry {
        at: instant("2024-05-01T08:00:00Z") + Duration::minutes(minute),
        kind: ActivityKind::DocumentAdded,
        subject: subject.to_owned(),
    }
}

#[test]
fn entries_come_back_newest_first_and_only_up_to_the_capacity() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("activity.yaml");
    let log = FileActivityLog::with_capacity(&path, 3);
    let entries: Vec<_> = (1..=4).map(|minute| entry(minute, &format!("file-{minute}.pdf"))).collect();

    for entry in &entries {
        log.record(entry).expect("recorded");
    }

    let expected: Vec<_> = entries.iter().rev().take(3).cloned().collect();
    assert_eq!(FileActivityLog::new(&path).recent(10).expect("entries"), expected);
    assert_eq!(log.recent(1).expect("entries"), [entries[3].clone()]);
}

#[test]
fn entries_of_an_unknown_kind_are_skipped_but_kept_in_the_file() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("activity.yaml");
    fs::write(
        &path,
        "entries:\n- at: 2024-05-01T08:00:00Z\n  kind: FUTURE_KIND\n  subject: a.pdf\n- at: 2024-05-01T07:00:00Z\n  kind: DOCUMENT_ADDED\n  subject: b.pdf\n",
    )
    .expect("file");
    let log = FileActivityLog::new(&path);

    assert_eq!(
        log.recent(10).expect("entries"),
        [ActivityEntry {
            at: instant("2024-05-01T07:00:00Z"),
            kind: ActivityKind::DocumentAdded,
            subject: "b.pdf".to_owned()
        }]
    );
    log.record(&entry(120, "c.pdf")).expect("recorded");
    assert!(read(&path).contains("kind: FUTURE_KIND"));
}

#[test]
fn the_log_is_written_as_jackson_wrote_it() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("activity.yaml");
    let log = FileActivityLog::new(&path);

    log.record(&ActivityEntry {
        at: instant("2024-05-01T08:00:00Z"),
        kind: ActivityKind::DocumentAdded,
        subject: "a: b.pdf".to_owned(),
    })
    .expect("recorded");
    log.record(&ActivityEntry {
        at: instant("2024-05-01T09:00:00.5Z"),
        kind: ActivityKind::PdfEdited,
        subject: "2024".to_owned(),
    })
    .expect("recorded");

    assert_eq!(read(&path), fixture("jackson/activity.yaml"));
}
