//! The presenters of the dashboard and the open tasks: pure functions from the catalog and the last dashboard
//! build to [`DashboardViewDto`] (Kotlin `DashboardViewModel.toUiState` and what its composables derived).

pub mod actions;
mod attention;
mod buttons;
pub mod context;
pub mod groups;
mod stats;
mod texts;

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use chrono::{DateTime, TimeZone, Utc};
use dms_application::library::{Dashboard, FolderState};
use dms_application::port::state::{ActivityEntry, ActivityKind};
use dms_domain::catalog::Catalog;
use dms_domain::document::DocumentId;
use dms_domain::library::{FolderRole, LibraryFolder, LibraryFolderId};
use dms_domain::task::OpenTask;

use self::context::TaskContext;
use crate::features::dashboard::dto::{
    ActivityDto, DashboardViewDto, FolderHealth, FolderStatusDto, TaskGroupDto, TaskGroupKind, TaskRowDto,
};
use crate::features::dashboard::format;

/// What the dashboard is built from. `dashboard` is the session's last build (`None` before the first one),
/// `foreign` the folders that hold another library, `scan_count` the scans without text found so far.
pub struct DashboardInput<'a, Tz: TimeZone> {
    pub catalog: &'a Catalog,
    pub dashboard: Option<&'a Dashboard>,
    pub foreign: &'a [LibraryFolderId],
    pub last_scan_at: Option<DateTime<Utc>>,
    pub ai_enabled: bool,
    pub scan_count: Option<usize>,
    pub now: DateTime<Utc>,
    pub zone: &'a Tz,
}

pub fn present_dashboard<Tz: TimeZone>(input: &DashboardInput<'_, Tz>) -> DashboardViewDto
where
    Tz::Offset: std::fmt::Display,
{
    let catalog = input.catalog;
    let folder_states: &[FolderState] = input.dashboard.map_or(&[], |dashboard| dashboard.folders.as_slice());
    let context = TaskContext::new(folder_states.iter().map(|state| &state.folder).collect(), catalog);
    let tasks: &[OpenTask] = input.dashboard.map_or(&[], |dashboard| dashboard.tasks.as_slice());
    let groups = present_groups(tasks, &context);
    let inbox = stats::inbox_of(catalog, input.ai_enabled);
    let by_year = stats::by_year(catalog, input.now, input.zone);
    let upcoming = input.dashboard.map_or(&[][..], |dashboard| dashboard.upcoming.as_slice());
    let (untagged, unclassified) = ai_candidates(catalog, tasks);
    DashboardViewDto {
        header_line: stats::header_line(catalog, folder_states.len(), input.last_scan_at, input.zone),
        stats: stats::stat_columns(catalog, inbox.as_ref(), folder_states.len(), &by_year, input.now),
        ai_band: stats::ai_band(input.ai_enabled, untagged, unclassified),
        attention: attention::attention(inbox.as_ref(), upcoming, &groups, input.scan_count, &context),
        recent: stats::recent(catalog, input.now, input.zone),
        primary_folder_id: context.primary_id().map(ToString::to_string),
        categories: stats::categories(catalog),
        tags: stats::tags(catalog),
        by_year,
        by_kind: stats::by_kind(catalog),
        correspondents: stats::correspondents(catalog),
        folders: folder_states.iter().map(|state| folder_status(state, input.foreign)).collect(),
        activity: input.dashboard.map_or_else(Vec::new, |dashboard| {
            dashboard.recent_activity.iter().map(|entry| activity(entry, input.zone)).collect()
        }),
        groups,
    }
}

/// The open tasks by group in the fixed group order; only groups with tasks.
pub fn present_groups(tasks: &[OpenTask], context: &TaskContext<'_>) -> Vec<TaskGroupDto> {
    groups::ORDER
        .into_iter()
        .filter_map(|group| {
            let members: Vec<&OpenTask> =
                tasks.iter().filter(|task| groups::group_of(task, context) == group).collect();
            (!members.is_empty()).then(|| group_dto(group, &members, context))
        })
        .collect()
}

fn group_dto(group: TaskGroupKind, tasks: &[&OpenTask], context: &TaskContext<'_>) -> TaskGroupDto {
    TaskGroupDto {
        group,
        label: groups::label(group).to_owned(),
        icon: groups::icon(group).to_owned(),
        rows: tasks.iter().map(|task| task_row(task, context)).collect(),
        bulk_actions: groups::bulk_buttons(group, tasks, context),
    }
}

fn task_row(task: &OpenTask, context: &TaskContext<'_>) -> TaskRowDto {
    let files = actions::files_for(task, context);
    // The copies of a duplicates task carry their own Open buttons.
    let copies = actions::copies_for(task, context);
    TaskRowDto {
        key: task.key().value().to_owned(),
        title: texts::title(task, context),
        detail: texts::detail(task, context),
        buttons: actions::buttons_for(task, context),
        opens: if copies.is_empty() { actions::opens_of(&files) } else { Vec::new() },
        preview: files.into_iter().next().map(|file| file.file),
        copies,
    }
}

/// Documents the AI can read with no suggestion waiting: (without tags, without classification).
fn ai_candidates(catalog: &Catalog, tasks: &[OpenTask]) -> (usize, usize) {
    let pending: HashSet<&DocumentId> = tasks
        .iter()
        .filter_map(|task| match task {
            OpenTask::ReviewSuggestion(suggestion) => Some(&suggestion.document_id),
            _ => None,
        })
        .collect();
    let candidates: Vec<_> = catalog
        .documents()
        .values()
        .filter(|document| document.kind().readable_by_ai() && !pending.contains(&document.id))
        .collect();
    (
        candidates.iter().filter(|document| document.tags.is_empty()).count(),
        candidates.iter().filter(|document| document.classification.is_none()).count(),
    )
}

fn folder_status(state: &FolderState, foreign: &[LibraryFolderId]) -> FolderStatusDto {
    let folder: &LibraryFolder = &state.folder;
    let health = if foreign.contains(&folder.id) {
        FolderHealth::Foreign
    } else {
        match state.reachable {
            None => FolderHealth::NotScanned,
            Some(true) => FolderHealth::Reachable,
            Some(false) => FolderHealth::Offline,
        }
    };
    let label = match health {
        FolderHealth::NotScanned => "not scanned yet",
        FolderHealth::Reachable => "reachable",
        FolderHealth::Offline => "offline",
        FolderHealth::Foreign => "holds another library, left alone",
    };
    let pending =
        if state.pending_operations > 0 { format!(", {} pending", state.pending_operations) } else { String::new() };
    let primary = folder.role == FolderRole::Primary;
    FolderStatusDto {
        title: if primary { format!("{} · primary", folder.display_name) } else { folder.display_name.clone() },
        location: folder.location.to_string(),
        primary,
        health,
        status: format!("{label}{pending}"),
    }
}

fn activity<Tz: TimeZone>(entry: &ActivityEntry, zone: &Tz) -> ActivityDto
where
    Tz::Offset: std::fmt::Display,
{
    ActivityDto { line: format!("{} {}", verb(entry.kind), entry.subject), time: format::day_and_time(entry.at, zone) }
}

fn verb(kind: ActivityKind) -> &'static str {
    match kind {
        ActivityKind::DocumentAdded => "Added",
        ActivityKind::DocumentMoved => "Moved",
        ActivityKind::DocumentTrashed => "Trashed",
        ActivityKind::DocumentRestored => "Restored",
        ActivityKind::FolderCreated => "Created folder",
        ActivityKind::FolderMoved => "Moved folder",
        ActivityKind::FolderTrashed => "Trashed folder",
        ActivityKind::CatalogEdited => "Edited catalog",
        ActivityKind::SuggestionApproved => "Approved suggestion",
        ActivityKind::ConflictResolved => "Resolved",
        ActivityKind::Synchronized => "Synchronized",
        ActivityKind::DocumentFiled => "Filed",
        ActivityKind::Restructured => "Restructured",
        ActivityKind::Exported => "Exported",
        ActivityKind::PdfEdited => "Edited PDF",
    }
}
