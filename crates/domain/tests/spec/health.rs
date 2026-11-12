//! The archive check's judgement, its schedule, the snapshot retention and what restoring a snapshot changes.

use chrono::{DateTime, TimeDelta, Utc};
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::health::{
    CopyObservation, CopyState, HealthFinding, HealthJudge, HealthSchedule, PartDiff, SnapshotDiff, SnapshotRetention,
};
use dms_domain::library::LibraryFolderId;

use crate::test_data::{self, Applied, catalog_with, document, document_with, hash, instant, tag};

fn folder(id: &str) -> LibraryFolderId {
    LibraryFolderId::new(id).expect("valid id")
}

fn read(folder_id: &str, seed: i32, opens: Option<bool>) -> CopyObservation {
    CopyObservation {
        folder_id: folder(folder_id),
        state: CopyState::Read { content_hash: hash(seed), size: 100, opens },
    }
}

fn state(folder_id: &str, state: CopyState) -> CopyObservation {
    CopyObservation { folder_id: folder(folder_id), state }
}

#[test]
fn a_copy_that_differs_while_another_matches_is_damaged_and_repaired_from_the_primary() {
    let doc = document_with("d1", "a.pdf", 1, 100);
    let copies = [read("primary", 1, Some(true)), read("cloud", 2, Some(true)), read("disk", 1, Some(true))];
    let findings = HealthJudge::judge(&doc, &copies, &folder("primary"));
    assert_eq!(
        findings,
        vec![HealthFinding::Damaged {
            document_id: doc.id.clone(),
            folder_id: folder("cloud"),
            source_folder_id: folder("primary"),
        }]
    );
}

#[test]
fn a_damaged_primary_is_repaired_from_the_first_matching_sync_folder() {
    let doc = document_with("d1", "a.pdf", 1, 100);
    let copies = [read("primary", 2, None), read("cloud", 1, None), read("disk", 1, None)];
    let findings = HealthJudge::judge(&doc, &copies, &folder("primary"));
    assert_eq!(
        findings,
        vec![HealthFinding::Damaged {
            document_id: doc.id.clone(),
            folder_id: folder("primary"),
            source_folder_id: folder("cloud"),
        }]
    );
}

#[test]
fn a_copy_that_cannot_be_read_is_damaged_and_a_file_no_folder_can_read_is_unreadable() {
    let doc = document_with("d1", "a.txt", 1, 100);
    let one_failed = [read("primary", 1, None), state("cloud", CopyState::ReadFailed)];
    assert_eq!(
        HealthJudge::judge(&doc, &one_failed, &folder("primary")),
        vec![HealthFinding::Damaged {
            document_id: doc.id.clone(),
            folder_id: folder("cloud"),
            source_folder_id: folder("primary"),
        }]
    );
    let all_failed = [state("primary", CopyState::ReadFailed), state("cloud", CopyState::ReadFailed)];
    assert_eq!(
        HealthJudge::judge(&doc, &all_failed, &folder("primary")),
        vec![HealthFinding::Unreadable { document_id: doc.id.clone() }]
    );
}

#[test]
fn copies_that_all_agree_but_differ_from_the_catalog_are_a_modification_not_a_finding() {
    let doc = document_with("d1", "a.pdf", 1, 100);
    let copies = [read("primary", 2, Some(true)), read("cloud", 2, Some(true))];
    assert_eq!(HealthJudge::judge(&doc, &copies, &folder("primary")), Vec::<HealthFinding>::new());
}

#[test]
fn copies_that_disagree_with_each_other_and_the_catalog_have_no_repair_source() {
    let doc = document_with("d1", "a.pdf", 1, 100);
    let copies = [read("primary", 2, None), read("cloud", 3, None)];
    assert_eq!(HealthJudge::judge(&doc, &copies, &folder("primary")), Vec::<HealthFinding>::new());
}

#[test]
fn missing_and_not_downloaded_copies_are_no_finding() {
    let doc = document_with("d1", "a.pdf", 1, 100);
    let copies =
        [read("primary", 1, Some(true)), state("cloud", CopyState::NotDownloaded), state("disk", CopyState::Missing)];
    assert_eq!(HealthJudge::judge(&doc, &copies, &folder("primary")), Vec::<HealthFinding>::new());
}

#[test]
fn a_pdf_that_opens_in_no_copy_is_unreadable_but_one_good_copy_is_enough() {
    let doc = document_with("d1", "a.pdf", 1, 100);
    let broken = [read("primary", 1, Some(false)), read("cloud", 1, Some(false))];
    assert_eq!(
        HealthJudge::judge(&doc, &broken, &folder("primary")),
        vec![HealthFinding::Unreadable { document_id: doc.id.clone() }]
    );
    let one_good = [read("primary", 1, Some(false)), read("cloud", 1, Some(true))];
    assert_eq!(HealthJudge::judge(&doc, &one_good, &folder("primary")), Vec::<HealthFinding>::new());
}

#[test]
fn the_schedule_is_due_after_its_interval_or_when_never_checked_and_never_when_off() {
    let now = instant("2026-11-11T12:00:00Z");
    let six_days = now - TimeDelta::days(6);
    let eight_days = now - TimeDelta::days(8);
    assert!(HealthSchedule::Weekly.is_due(None, now));
    assert!(!HealthSchedule::Weekly.is_due(Some(six_days), now));
    assert!(HealthSchedule::Weekly.is_due(Some(eight_days), now));
    assert!(!HealthSchedule::Monthly.is_due(Some(eight_days), now));
    assert!(HealthSchedule::Monthly.is_due(Some(now - TimeDelta::days(31)), now));
    assert!(!HealthSchedule::Off.is_due(None, now));
    assert_eq!(HealthSchedule::default(), HealthSchedule::Weekly);
}

fn hours_ago(now: DateTime<Utc>, hours: &[i64]) -> Vec<DateTime<Utc>> {
    hours.iter().map(|hours| now - TimeDelta::hours(*hours)).collect()
}

#[test]
fn the_newest_twenty_snapshots_are_always_kept() {
    let now = instant("2026-11-11T12:00:00Z");
    let taken: Vec<DateTime<Utc>> = (0..20).map(|minutes| now - TimeDelta::minutes(minutes)).collect();
    assert_eq!(SnapshotRetention::to_remove(&taken, now), Vec::<usize>::new());
}

#[test]
fn older_snapshots_keep_one_per_day_for_a_month_and_one_per_week_for_a_year() {
    let now = instant("2026-11-11T12:00:00Z");
    // 20 recent ones within the last hour, then two on one day 10 days ago, two in one week 100 days ago and one
    // two years ago.
    let mut hours: Vec<i64> = vec![0; 20];
    hours.extend([240, 241, 2400, 2401, 17_520]);
    let mut taken = hours_ago(now, &hours);
    for (index, at) in taken.iter_mut().enumerate().take(20) {
        *at -= TimeDelta::seconds(i64::try_from(index).expect("small"));
    }
    let removed = SnapshotRetention::to_remove(&taken, now);
    // Of each pair the newer one stays (index 20 and 22); the two-year-old one goes.
    assert_eq!(removed, vec![21, 23, 24]);
}

#[test]
fn a_diff_counts_what_restoring_brings_back_drops_and_changes() {
    let a = document("d1", "a.pdf");
    let b = document("d2", "b.pdf");
    let c = document("d3", "c.pdf");
    let tags = std::iter::once(tag("Old")).collect();
    let snapshot = catalog_with(&[a.clone(), b]).set_tags(&test_data::document_id("d1"), &tags).applied();
    let current = catalog_with(&[a, c]);
    let diff = SnapshotDiff::between(&current, &snapshot);
    assert_eq!(diff.documents, PartDiff { added: 1, removed: 1, changed: 1 });
    assert_eq!(diff.tags, PartDiff { added: 1, removed: 0, changed: 0 });
    assert!(!diff.is_empty());
    assert!(SnapshotDiff::between(&current, &current).is_empty());
}

#[test]
fn restoring_a_snapshot_takes_its_content_as_the_next_revision() {
    let snapshot = catalog_with(&[document("d1", "a.pdf")]);
    let current = snapshot.add_document(document("d2", "b.pdf")).applied();
    let CatalogUpdate::Applied(restored) = current.restored_from(&snapshot).expect("same library") else {
        panic!("applied");
    };
    assert_eq!(restored.revision(), current.revision().next());
    assert_eq!(restored.documents(), snapshot.documents());
}

#[test]
fn a_snapshot_of_another_library_is_refused() {
    let other = Catalog::empty(LibraryId::new("other").expect("id"));
    assert!(catalog_with(&[]).restored_from(&other).is_err());
}
