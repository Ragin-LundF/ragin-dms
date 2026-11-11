//! The dashboard's commands: its view, the answers to open tasks, the AI runs and the small helpers its rows need.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Local, Utc};
use dms_application::pdf::model::ScansOutcome;
use dms_domain::document::DocumentId;
use dms_domain::library::{LibraryFolderId, RelativePath};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use super::ai_batch::{self, AiRun};
use super::dto::{DashboardAction, DashboardViewDto};
use super::plan::plan;
use super::run::{execute, outcome_of};
use super::view::{DashboardInput, present_dashboard};
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::messages;
use crate::session::reporting::{blocking, busy};
use crate::session::{Lane, LibraryStatus, UiMessage};

/// Everything the dashboard and the open tasks screen show. `scan_count` is the number of scans without text the
/// window found so far (`None` while it did not look).
#[tauri::command]
#[specta::specta]
pub fn dashboard_view(ctx: AppState<'_>, scan_count: Option<u32>) -> Result<DashboardViewDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let dashboard = ctx.dashboard();
    let state = ctx.session.state();
    let foreign: Vec<LibraryFolderId> = match &state.status {
        LibraryStatus::Open { foreign_folders, .. } => {
            foreign_folders.iter().filter_map(|id| LibraryFolderId::new(id.as_str()).ok()).collect()
        }
        _ => Vec::new(),
    };
    let last_scan_at = state
        .last_scan_at
        .as_deref()
        .and_then(|at| DateTime::parse_from_rfc3339(at).ok())
        .map(|at| at.with_timezone(&Utc));
    Ok(present_dashboard(&DashboardInput {
        catalog: &catalog,
        dashboard: dashboard.as_ref(),
        foreign: &foreign,
        last_scan_at,
        ai_enabled: state.ai_enabled,
        scan_count: scan_count.map(|count| count as usize),
        now: Utc::now(),
        zone: &Local,
    }))
}

/// Answers an open task (or all tasks of a group) as one operation; it reports its own result. Actions that only
/// open or navigate are handled by the window.
#[tauri::command]
#[specta::specta]
pub fn run_dashboard_action(ctx: AppState<'_>, action: DashboardAction) -> Result<(), IpcError> {
    if action == DashboardAction::AnalyzeInbox {
        return ctx.analyze_inbox();
    }
    let services = Arc::clone(ctx.require_services()?);
    let catalog = ctx.require_catalog()?;
    let dashboard = ctx.dashboard().ok_or_else(IpcError::not_open)?;
    let plan = plan(&action, &dashboard, &catalog)?;
    let context = Arc::clone(&ctx);
    execute(&ctx.session, plan, move |call| outcome_of(&services, call), move |change| context.forget(change))
}

/// Tags every document without tags with the AI (AI lane, cancellable).
#[tauri::command]
#[specta::specta]
pub fn ai_tag_all(ctx: AppState<'_>) -> Result<(), IpcError> {
    let services = Arc::clone(ctx.require_services()?);
    start_ai(&ctx, ai_batch::TAG_ALL, move |progress| services.ai.tag_all_untagged.run(progress))
}

/// Classifies every document without a classification with the AI (AI lane, cancellable).
#[tauri::command]
#[specta::specta]
pub fn ai_classify_all(ctx: AppState<'_>) -> Result<(), IpcError> {
    let services = Arc::clone(ctx.require_services()?);
    start_ai(&ctx, ai_batch::CLASSIFY_ALL, move |progress| {
        services.ai.classify_all_new.run(&RelativePath::root(), progress)
    })
}

fn start_ai<B>(ctx: &Arc<AppContext>, run: AiRun, batch: B) -> Result<(), IpcError>
where
    B: FnOnce(
            &dyn dms_application::common::Progress<dms_application::classification::ClassificationProgress>,
        ) -> Option<dms_application::classification::ClassificationProgress>
        + Send
        + 'static,
{
    let context = Arc::clone(ctx);
    let name_of = move |id: &DocumentId| {
        context
            .catalog()
            .and_then(|catalog| catalog.documents().get(id).map(|document| document.path.to_string()))
            .unwrap_or_else(|| id.to_string())
    };
    ai_batch::start(&ctx.session, run, batch, name_of)
}

/// The PDFs without a text layer (document ids); only reads, in the background.
#[tauri::command]
#[specta::specta]
pub async fn find_scans_without_text(ctx: AppState<'_>) -> Result<Vec<String>, IpcError> {
    let services = Arc::clone(ctx.require_services()?);
    blocking(move || {
        Ok(match services.pdf.scans_without_text.run() {
            ScansOutcome::Found(ids) => ids.iter().map(ToString::to_string).collect(),
            ScansOutcome::LibraryNotOpen => Vec::new(),
        })
    })
    .await
}

/// Writes the upcoming deadlines and retention dates as `deadlines.ics` into `directory`.
#[tauri::command]
#[specta::specta]
pub fn export_calendar(ctx: AppState<'_>, directory: String) -> Result<(), IpcError> {
    let services = Arc::clone(ctx.require_services()?);
    let catalog = ctx.require_catalog()?;
    if !Path::new(&directory).is_absolute() {
        return Err(IpcError::invalid("expected an absolute path"));
    }
    let calendar = dms_application::deadline::DeadlineCalendar::of(&catalog, Local::now().date_naive());
    let label = "Saving the calendar";
    let started = ctx.session.perform(label, Lane::Library, false, move |operation| {
        let message = match services.organize.export_writer.write_text(&directory, "deadlines.ics", &calendar) {
            Ok(path) => UiMessage::info(format!("Saved {path}")),
            Err(error) => {
                UiMessage::error(format!("The calendar could not be saved: {}", messages::failure(error.reason)))
            }
        };
        operation.session().report(message);
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, label)) }
}

/// Opens the file at `path` in library folder `folder_id` in its default application, to see what a task is about;
/// reports "No application could open {name}" when that fails.
#[tauri::command]
#[specta::specta]
pub fn open_task_file(app: AppHandle, ctx: AppState<'_>, folder_id: String, path: String) -> Result<(), IpcError> {
    let relative =
        dms_domain::library::RelativePath::of(&path).map_err(|error| IpcError::invalid(error.to_string()))?;
    let state = ctx.session.state();
    let Some(folder) = state.folders.iter().find(|folder| folder.id == folder_id) else {
        return Ok(());
    };
    let file = relative.segments().iter().fold(Path::new(&folder.location).to_path_buf(), |file, part| file.join(part));
    let opened = file.is_file() && app.opener().open_path(file.to_string_lossy(), None::<&str>).is_ok();
    if !opened {
        ctx.session.report(UiMessage::error(format!("No application could open {}", relative.file_name())));
    }
    Ok(())
}
