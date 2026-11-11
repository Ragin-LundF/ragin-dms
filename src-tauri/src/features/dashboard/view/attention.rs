//! "Needs your attention": the inbox, the next dates, the open tasks and the scans without text in one list,
//! most urgent first, cut to five rows (Kotlin `AttentionRows`).

use dms_domain::deadline::UpcomingDate;
use dms_domain::iso_date;

use super::buttons::{ButtonExt, button};
use super::context::TaskContext;
use super::groups::{icon, severity_of};
use super::stats::{InboxSummary, count};
use crate::features::dashboard::dto::{
    AttentionDto, AttentionRowDto, AttentionSeverity, DashboardAction, DashboardButtonDto, DashboardButtonStyle,
    TaskFileDto, TaskGroupDto,
};
use crate::session::Lane;

pub const LIMIT: usize = 5;

/// `scan_count` is the number of PDFs without a text layer, or `None` while nobody looked for them.
pub fn attention(
    inbox: Option<&InboxSummary>,
    upcoming: &[UpcomingDate],
    groups: &[TaskGroupDto],
    scan_count: Option<usize>,
    context: &TaskContext<'_>,
) -> AttentionDto {
    let mut all: Vec<AttentionRowDto> = inbox.and_then(inbox_row).into_iter().collect();
    all.extend(upcoming.iter().map(|date| upcoming_row(date, context)));
    all.extend(task_rows(groups));
    all.extend(scan_count.and_then(scans_row));
    let total = count(all.len());
    // Stable, so rows of the same severity keep their build order.
    all.sort_by_key(|row| severity_rank(row.severity));
    all.truncate(LIMIT);
    AttentionDto { rows: all, total, header_buttons: header_buttons(upcoming, scan_count) }
}

fn severity_rank(severity: AttentionSeverity) -> u8 {
    match severity {
        AttentionSeverity::Passed => 0,
        AttentionSeverity::Soon => 1,
        AttentionSeverity::Info => 2,
    }
}

pub fn severity_label(severity: AttentionSeverity) -> &'static str {
    match severity {
        AttentionSeverity::Passed => "Date passed",
        AttentionSeverity::Soon => "Decide soon",
        AttentionSeverity::Info => "To do",
    }
}

fn row(key: String, severity: AttentionSeverity, title: String, detail: &str, icon: &str) -> AttentionRowDto {
    let label = severity_label(severity);
    AttentionRowDto {
        key,
        severity,
        title,
        detail: if detail.is_empty() { label.to_owned() } else { format!("{label} · {detail}") },
        icon: icon.to_owned(),
        preview: None,
        buttons: Vec::new(),
        opens: Vec::new(),
        copies: Vec::new(),
    }
}

fn inbox_row(inbox: &InboxSummary) -> Option<AttentionRowDto> {
    if inbox.count == 0 {
        return None;
    }
    let title = if inbox.count == 1 {
        "1 new document in the inbox".to_owned()
    } else {
        format!("{} new documents in the inbox", inbox.count)
    };
    let detail = format!("In {} of your library folders", inbox.path);
    let mut row = row("inbox".to_owned(), AttentionSeverity::Info, title, &detail, "move_to_inbox");
    if inbox.ai_analysis {
        row.buttons.push(
            button("Analyze all", DashboardAction::AnalyzeInbox)
                .icon("auto_awesome")
                .style(DashboardButtonStyle::Tonal)
                .waits_for(Lane::Ai),
        );
    }
    row.opens.push(button("Open inbox", DashboardAction::OpenInbox).icon("arrow_forward"));
    Some(row)
}

fn upcoming_row(upcoming: &UpcomingDate, context: &TaskContext<'_>) -> AttentionRowDto {
    let document = context.catalog().documents().get(&upcoming.document_id);
    let what = upcoming.field.as_ref().map_or_else(|| "Retention ends".to_owned(), |key| context.field_label(key));
    let title = document.map_or_else(
        || upcoming.document_id.to_string(),
        |document| document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned()),
    );
    let date = iso_date(upcoming.date);
    let key = format!("upcoming:{}:{what}:{date}", upcoming.document_id);
    let mut row = row(key, AttentionSeverity::Soon, title, &format!("{what} on {date}"), "event");
    row.preview = match (context.primary_id(), document) {
        (Some(primary), Some(document)) => {
            Some(TaskFileDto { folder_id: primary.to_string(), path: document.path.to_string() })
        }
        _ => None,
    };
    row.opens.push(button("Open", DashboardAction::OpenDocument { document_id: upcoming.document_id.to_string() }));
    row
}

fn task_rows(groups: &[TaskGroupDto]) -> Vec<AttentionRowDto> {
    groups
        .iter()
        .flat_map(|group| {
            group.rows.iter().map(|task| {
                let severity = severity_of(group.group);
                let mut row =
                    row(format!("task:{}", task.key), severity, task.title.clone(), &task.detail, icon(group.group));
                row.preview.clone_from(&task.preview);
                row.buttons.clone_from(&task.buttons);
                row.opens.clone_from(&task.opens);
                row.copies.clone_from(&task.copies);
                row
            })
        })
        .collect()
}

fn scans_row(scans: usize) -> Option<AttentionRowDto> {
    if scans == 0 {
        return None;
    }
    let title = if scans == 1 { "1 scan without text".to_owned() } else { format!("{scans} scans without text") };
    let mut row = row(
        "scans".to_owned(),
        AttentionSeverity::Info,
        title,
        "PDFs search cannot read, until they get a text layer",
        "document_scanner",
    );
    row.buttons = vec![
        button(format!("Make {scans} searchable"), DashboardAction::MakeScansSearchable)
            .style(DashboardButtonStyle::Tonal)
            .waits_for(Lane::Library),
        button("Look again", DashboardAction::FindScans).style(DashboardButtonStyle::Text).waits_for(Lane::Library),
    ];
    Some(row)
}

/// Looking for scans and saving the calendar belong to no single row, so they sit in the header.
fn header_buttons(upcoming: &[UpcomingDate], scan_count: Option<usize>) -> Vec<DashboardButtonDto> {
    let mut buttons = Vec::new();
    if scan_count.is_none_or(|scans| scans == 0) {
        buttons.push(
            button("Find scans without text", DashboardAction::FindScans)
                .style(DashboardButtonStyle::Text)
                .waits_for(Lane::Library),
        );
    }
    if !upcoming.is_empty() {
        buttons.push(button("Export .ics", DashboardAction::ExportCalendar).style(DashboardButtonStyle::Text));
    }
    buttons
}
