use chrono::Utc;
use dms_domain::change::MissingFile;
use dms_domain::deadline::UpcomingDate;
use dms_domain::field::FieldKey;

use super::*;
use crate::features::dashboard::dto::{AttentionSeverity, DashboardAction};
use crate::features::dashboard::view::DashboardInput;
use crate::session::Lane;

fn missing(document: &str) -> OpenTask {
    OpenTask::RestoreOrDelete(MissingFile {
        document_id: id(document),
        path: path(&format!("{document}.pdf")),
        missing_in: std::iter::once(folder_id(SYNC)).collect(),
        present_in: std::iter::once(folder_id(PRIMARY)).collect(),
    })
}

fn view_with(library: &Library, built: &Dashboard, scan_count: Option<usize>) -> DashboardViewDto {
    present_dashboard(&DashboardInput {
        catalog: library.catalog(),
        dashboard: Some(built),
        foreign: &[],
        last_scan_at: None,
        ai_enabled: true,
        scan_count,
        now: now(),
        zone: &Utc,
    })
}

#[test]
fn rows_are_sorted_by_severity_cut_to_five_and_the_total_counts_all() {
    let library = Library::new().inbox("Inbox", true).document("in", "Inbox/scan.pdf", "2026-03-01T00:00:00Z", nothing);
    let date = dms_domain::parse_iso_date("2026-02-01").expect("date");
    let mut tasks: Vec<OpenTask> = (1..=5).map(|index| missing(&format!("m{index}"))).collect();
    // A deadline still ahead within its lead time reads "Date passed" too.
    tasks.push(OpenTask::DeadlineDue { document_id: id("in"), field_key: FieldKey::new("due").expect("key"), date });
    let built = dashboard(tasks);
    let attention = view_with(&library, &built, Some(2)).attention;
    assert_eq!(attention.total, 8);
    assert_eq!(attention.rows.len(), 5);
    assert_eq!(attention.rows[0].severity, AttentionSeverity::Passed);
    assert!(attention.rows[0].detail.starts_with("Date passed · Dismiss once it is handled"));
    assert!(attention.rows[1..].iter().all(|row| row.severity == AttentionSeverity::Soon));
    assert_eq!(attention.rows[1].key, "task:missing:m1");
}

#[test]
fn the_inbox_row_offers_analysis_only_with_ai_analysis_on() {
    let library = Library::new().inbox("Inbox", true).document("in", "Inbox/scan.pdf", "2026-03-01T00:00:00Z", nothing);
    let attention = view_with(&library, &dashboard(Vec::new()), None).attention;
    let inbox = &attention.rows[0];
    assert_eq!(inbox.title, "1 new document in the inbox");
    assert_eq!(inbox.detail, "To do · In Inbox of your library folders");
    assert_eq!(labels(&inbox.buttons), vec!["Analyze all"]);
    assert_eq!(inbox.buttons[0].lane, Some(Lane::Ai));
    assert_eq!(inbox.opens[0].action, DashboardAction::OpenInbox);
    let without_ai =
        Library::new().inbox("Inbox", false).document("in", "Inbox/scan.pdf", "2026-03-01T00:00:00Z", nothing);
    assert!(view_with(&without_ai, &dashboard(Vec::new()), None).attention.rows[0].buttons.is_empty());
}

#[test]
fn scans_and_the_calendar_decide_the_header_buttons() {
    let library = Library::new().document("d1", "a.pdf", "2026-03-01T00:00:00Z", nothing);
    let mut built = dashboard(Vec::new());
    assert_eq!(labels(&view_with(&library, &built, None).attention.header_buttons), vec!["Find scans without text"]);
    built.upcoming = vec![UpcomingDate {
        document_id: id("d1"),
        field: None,
        date: dms_domain::parse_iso_date("2026-04-01").expect("date"),
    }];
    let attention = view_with(&library, &built, Some(3)).attention;
    assert_eq!(labels(&attention.header_buttons), vec!["Export .ics"]);
    let upcoming = attention.rows.iter().find(|row| row.icon == "event").expect("the date is listed");
    assert_eq!(upcoming.title, "a.pdf");
    assert_eq!(upcoming.detail, "Decide soon · Retention ends on 2026-04-01");
    let scans = attention.rows.iter().find(|row| row.key == "scans").expect("the scans are listed");
    assert_eq!(scans.title, "3 scans without text");
    assert_eq!(labels(&scans.buttons), vec!["Make 3 searchable", "Look again"]);
}

#[test]
fn nothing_to_do_shows_no_rows() {
    let attention = present(Library::new().catalog(), None).attention;
    assert_eq!(attention.total, 0);
    assert!(attention.rows.is_empty());
}
