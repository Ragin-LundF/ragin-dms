//! The presenters of Settings › Archive health: pure functions from the report, the catalog and the snapshots.

use chrono::{DateTime, Utc};
use dms_application::port::health::{CatalogSnapshot, FolderCheck, HealthReport};
use dms_domain::catalog::Catalog;
use dms_domain::health::{HealthFinding, HealthSchedule, PartDiff, SnapshotDiff};
use dms_domain::library::{FolderRole, LibraryFolder};

use super::dto::{HealthFolderDto, HealthScheduleDto, HealthSnapshotDto, HealthStateDto, HealthToneDto};
use crate::features::dashboard::dto::TaskRowDto;

pub struct HealthInput<'a> {
    pub catalog: &'a Catalog,
    pub folders: &'a [LibraryFolder],
    /// The report of this library, if it was ever checked on this device.
    pub report: Option<&'a HealthReport>,
    pub schedule: HealthSchedule,
    pub snapshots: &'a [CatalogSnapshot],
    pub findings: Vec<TaskRowDto>,
}

pub fn present_health(input: HealthInput<'_>) -> HealthStateDto {
    let report = input.report;
    let checks: &[FolderCheck] = report.map_or(&[], |report| report.folders.as_slice());
    let (headline, tone) = headline(report, input.findings.len());
    let files_checked = checks.iter().map(|check| check.checked).sum();
    let documents = u32::try_from(input.catalog.documents().len()).unwrap_or(u32::MAX);
    let expected: u32 = checks.iter().filter(|check| !check.offline).map(|_| documents).sum();
    let processed: u32 = checks.iter().map(|check| check.checked + check.not_downloaded + check.missing).sum();
    HealthStateDto {
        headline,
        tone,
        files_checked,
        #[allow(clippy::cast_precision_loss)] // a share for a ring
        done: if expected == 0 { 0.0 } else { (processed as f32 / expected as f32).min(1.0) },
        last_check_at: report.and_then(|report| report.finished_at).map(|at| at.to_rfc3339()),
        unfinished: report.is_some_and(|report| report.resume_after.is_some()),
        schedule: schedule_dto(input.schedule),
        folders: input.folders.iter().map(|folder| folder_row(folder, checks, report)).collect(),
        findings: input.findings,
        snapshots: snapshot_rows(input.snapshots, input.catalog),
    }
}

fn headline(report: Option<&HealthReport>, findings: usize) -> (String, HealthToneDto) {
    match (report.and_then(|report| report.finished_at), findings) {
        (_, 1) => ("1 problem found".to_owned(), HealthToneDto::Error),
        (_, count) if count > 1 => (format!("{count} problems found"), HealthToneDto::Error),
        (None, _) => ("Not checked yet".to_owned(), HealthToneDto::Neutral),
        (Some(_), _) => ("All intact".to_owned(), HealthToneDto::Good),
    }
}

fn folder_row(folder: &LibraryFolder, checks: &[FolderCheck], report: Option<&HealthReport>) -> HealthFolderDto {
    let check = checks.iter().find(|check| check.folder_id == folder.id);
    let damaged = report.map_or(0, |report| {
        report
            .findings
            .iter()
            .filter(|finding| matches!(finding, HealthFinding::Damaged { folder_id, .. } if *folder_id == folder.id))
            .count()
    });
    let (pill, tone) = match check {
        None => ("Not checked".to_owned(), HealthToneDto::Neutral),
        Some(check) if check.offline => ("Offline · next time".to_owned(), HealthToneDto::Neutral),
        Some(_) if damaged > 0 => (format!("{damaged} damaged"), HealthToneDto::Error),
        Some(check) if check.missing > 0 => (format!("{} missing", check.missing), HealthToneDto::Warning),
        Some(_) => ("Intact".to_owned(), HealthToneDto::Good),
    };
    HealthFolderDto {
        name: folder.display_name.clone(),
        location: folder.location.value().to_owned(),
        primary: folder.role == FolderRole::Primary,
        summary: check.map_or_else(String::new, summary),
        pill,
        tone,
    }
}

fn summary(check: &FolderCheck) -> String {
    let mut parts = vec![format!("{} files checked", check.checked)];
    if check.not_downloaded > 0 {
        parts.push(format!("{} not downloaded, skipped", check.not_downloaded));
    }
    parts.join(" · ")
}

fn snapshot_rows(snapshots: &[CatalogSnapshot], catalog: &Catalog) -> Vec<HealthSnapshotDto> {
    snapshots
        .iter()
        .map(|snapshot| HealthSnapshotDto {
            id: snapshot.id.value().to_owned(),
            taken_at: snapshot.taken_at.to_rfc3339(),
            label: format!("Revision {}", snapshot.revision),
            current: snapshot.revision == catalog.revision(),
        })
        .collect()
}

pub fn schedule_dto(schedule: HealthSchedule) -> HealthScheduleDto {
    match schedule {
        HealthSchedule::Off => HealthScheduleDto::Off,
        HealthSchedule::Weekly => HealthScheduleDto::Weekly,
        HealthSchedule::Monthly => HealthScheduleDto::Monthly,
    }
}

pub fn schedule_of(dto: HealthScheduleDto) -> HealthSchedule {
    match dto {
        HealthScheduleDto::Off => HealthSchedule::Off,
        HealthScheduleDto::Weekly => HealthSchedule::Weekly,
        HealthScheduleDto::Monthly => HealthSchedule::Monthly,
    }
}

/// What restoring changes, in words; "Nothing would change" when it is the same.
pub fn diff_lines(diff: &SnapshotDiff) -> Vec<String> {
    if diff.is_empty() {
        return vec!["Nothing would change".to_owned()];
    }
    let mut lines = Vec::new();
    for (part, one, many) in [
        (diff.documents, "document", "documents"),
        (diff.categories, "category", "categories"),
        (diff.tags, "tag", "tags"),
        (diff.contracts, "contract", "contracts"),
        (diff.cases, "case", "cases"),
    ] {
        lines.extend(part_lines(part, one, many));
    }
    if diff.settings_changed {
        lines.push("The library settings go back to that state".to_owned());
    }
    lines
}

fn part_lines(part: PartDiff, one: &str, many: &str) -> Vec<String> {
    let noun = |count: usize| if count == 1 { one } else { many };
    let mut lines = Vec::new();
    if part.added > 0 {
        let verb = if part.added == 1 { "comes" } else { "come" };
        lines.push(format!("{} {} {verb} back", part.added, noun(part.added)));
    }
    if part.removed > 0 {
        let verb = if part.removed == 1 { "leaves" } else { "leave" };
        lines.push(format!("{} {} {verb} the catalog", part.removed, noun(part.removed)));
    }
    if part.changed > 0 {
        lines.push(format!("{} {} go back to their earlier values", part.changed, noun(part.changed)));
    }
    lines
}

/// Whether the check should start on its own: an unfinished pass continues, otherwise the schedule decides.
pub fn check_due(schedule: HealthSchedule, report: Option<&HealthReport>, now: DateTime<Utc>) -> bool {
    if schedule == HealthSchedule::Off {
        return false;
    }
    report.is_some_and(|report| report.resume_after.is_some())
        || schedule.is_due(report.and_then(|report| report.finished_at), now)
}

#[cfg(test)]
mod tests {
    use dms_application::port::health::CatalogSnapshotId;
    use dms_domain::catalog::{CatalogRevision, LibraryId};
    use dms_domain::document::DocumentId;
    use dms_domain::library::{FolderLocation, LibraryFolderId};

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        text.parse().expect("instant")
    }

    fn folder(id: &str, role: FolderRole) -> LibraryFolder {
        LibraryFolder::new(
            LibraryFolderId::new(id).expect("id"),
            id,
            FolderLocation::new(format!("/library/{id}")).expect("location"),
            role,
        )
        .expect("folder")
    }

    fn report() -> HealthReport {
        let mut report = HealthReport::new(LibraryId::new("library").expect("id"));
        report.finished_at = Some(at("2026-11-10T03:12:00Z"));
        report.folders = vec![
            FolderCheck {
                checked: 4,
                not_downloaded: 1,
                ..FolderCheck::new(LibraryFolderId::new("primary").expect("id"))
            },
            FolderCheck { checked: 5, ..FolderCheck::new(LibraryFolderId::new("cloud").expect("id")) },
            FolderCheck { offline: true, ..FolderCheck::new(LibraryFolderId::new("disk").expect("id")) },
        ];
        report.findings = vec![HealthFinding::Damaged {
            document_id: DocumentId::new("d1").expect("id"),
            folder_id: LibraryFolderId::new("cloud").expect("id"),
            source_folder_id: LibraryFolderId::new("primary").expect("id"),
        }];
        report
    }

    #[test]
    fn folders_show_damage_first_then_offline_and_skipped_files() {
        let catalog = Catalog::empty(LibraryId::new("library").expect("id"));
        let folders = [
            folder("primary", FolderRole::Primary),
            folder("cloud", FolderRole::Sync),
            folder("disk", FolderRole::Sync),
        ];
        let report = report();
        let view = present_health(HealthInput {
            catalog: &catalog,
            folders: &folders,
            report: Some(&report),
            schedule: HealthSchedule::Monthly,
            snapshots: &[],
            findings: Vec::new(),
        });
        let pills: Vec<(&str, HealthToneDto)> = view.folders.iter().map(|row| (row.pill.as_str(), row.tone)).collect();
        assert_eq!(
            pills,
            vec![
                ("Intact", HealthToneDto::Good),
                ("1 damaged", HealthToneDto::Error),
                ("Offline · next time", HealthToneDto::Neutral)
            ]
        );
        assert_eq!(view.folders[0].summary, "4 files checked · 1 not downloaded, skipped");
        assert_eq!(view.files_checked, 9);
        assert_eq!(view.schedule, HealthScheduleDto::Monthly);
        assert_eq!(view.headline, "All intact", "dismissed findings are no rows");
    }

    #[test]
    fn a_library_never_checked_says_so() {
        let catalog = Catalog::empty(LibraryId::new("library").expect("id"));
        let snapshot = CatalogSnapshot {
            id: CatalogSnapshotId::new("20261111T091400Z-r0").expect("id"),
            taken_at: at("2026-11-11T09:14:00Z"),
            revision: CatalogRevision::INITIAL,
        };
        let view = present_health(HealthInput {
            catalog: &catalog,
            folders: &[folder("primary", FolderRole::Primary)],
            report: None,
            schedule: HealthSchedule::Weekly,
            snapshots: &[snapshot],
            findings: Vec::new(),
        });
        assert_eq!((view.headline.as_str(), view.tone), ("Not checked yet", HealthToneDto::Neutral));
        assert_eq!(view.folders[0].pill, "Not checked");
        assert!(view.snapshots[0].current);
    }

    #[test]
    fn a_diff_reads_as_what_comes_back_and_what_leaves() {
        let diff = SnapshotDiff {
            documents: PartDiff { added: 2, removed: 1, changed: 0 },
            tags: PartDiff { added: 1, removed: 0, changed: 0 },
            ..SnapshotDiff::default()
        };
        assert_eq!(
            diff_lines(&diff),
            vec!["2 documents come back", "1 document leaves the catalog", "1 tag comes back"]
        );
        assert_eq!(diff_lines(&SnapshotDiff::default()), vec!["Nothing would change"]);
    }

    #[test]
    fn the_check_starts_on_its_own_when_due_or_unfinished_but_never_when_off() {
        let now = at("2026-11-11T12:00:00Z");
        let mut recent = report();
        assert!(!check_due(HealthSchedule::Weekly, Some(&recent), now));
        assert!(check_due(HealthSchedule::Weekly, None, now));
        recent.resume_after = Some(DocumentId::new("d1").expect("id"));
        assert!(check_due(HealthSchedule::Weekly, Some(&recent), now));
        assert!(!check_due(HealthSchedule::Off, Some(&recent), now));
    }
}
