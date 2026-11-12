//! Commands of Settings › Archive health and the dashboard card.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;

use dms_application::health::SnapshotComparison;
use dms_application::port::health::CatalogSnapshotId;
use dms_application::port::state::Preferences;
use dms_domain::task::OpenTask;

use super::check::report_of_open_library;
use super::dto::{HealthScheduleDto, HealthSnapshotDiffDto, HealthStateDto};
use super::view::{self, HealthInput};
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::dashboard::dto::TaskRowDto;
use crate::features::dashboard::view::context::TaskContext;
use crate::features::dashboard::view::present_groups;
use crate::messages;
use crate::session::Lane;
use crate::session::reporting::{blocking, busy};

/// The archive health of the open library: last check, schedule, folders, findings and snapshots.
#[tauri::command]
#[specta::specta]
pub async fn health_state(ctx: AppState<'_>) -> Result<HealthStateDto, IpcError> {
    let app = Arc::clone(&ctx);
    blocking(move || {
        let services = app.require_services()?;
        let catalog = app.require_catalog()?;
        let folders = services.library.settings.load_library_folders().ok().flatten();
        let schedule = services.library.settings.load_preferences().unwrap_or_default().health_check;
        let report = report_of_open_library(services);
        let snapshots = services.sync.snapshots.list(catalog.library_id()).unwrap_or_default();
        Ok(view::present_health(HealthInput {
            catalog: &catalog,
            folders: folders.as_ref().map_or(&[], |folders| folders.folders()),
            report: report.as_ref(),
            schedule,
            snapshots: &snapshots,
            findings: finding_rows(&app),
        }))
    })
    .await
}

/// The health tasks of the dashboard, with the same buttons as there.
fn finding_rows(ctx: &AppContext) -> Vec<TaskRowDto> {
    let (Some(dashboard), Some(catalog)) = (ctx.dashboard(), ctx.catalog()) else { return Vec::new() };
    let context = TaskContext::new(dashboard.folders.iter().map(|state| &state.folder).collect(), &catalog);
    let tasks: Vec<OpenTask> = dashboard
        .tasks
        .into_iter()
        .filter(|task| matches!(task, OpenTask::DamagedCopy { .. } | OpenTask::UnreadableDocument { .. }))
        .collect();
    present_groups(&tasks, &context).into_iter().flat_map(|group| group.rows).collect()
}

/// "Check now": starts or continues the archive check.
#[tauri::command]
#[specta::specta]
pub fn health_check_now(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.check_archive(false)
}

/// How often this device checks the archive on its own; saved as the operation "Saving the preferences".
#[tauri::command]
#[specta::specta]
pub fn health_set_schedule(ctx: AppState<'_>, schedule: HealthScheduleDto) -> Result<(), IpcError> {
    let health_check = view::schedule_of(schedule);
    ctx.update_preferences(move |preferences| Preferences { health_check, ..preferences })
}

/// What restoring `snapshot_id` would change.
#[tauri::command]
#[specta::specta]
pub async fn health_compare_snapshot(
    ctx: AppState<'_>,
    snapshot_id: String,
) -> Result<HealthSnapshotDiffDto, IpcError> {
    let id = snapshot(&snapshot_id)?;
    let services = Arc::clone(ctx.require_services()?);
    blocking(move || match services.sync.compare_catalog_snapshot.run(&id) {
        SnapshotComparison::Compared(diff) => Ok(HealthSnapshotDiffDto { lines: view::diff_lines(&diff) }),
        SnapshotComparison::LibraryNotOpen => Err(IpcError::not_open()),
        SnapshotComparison::Failed(error) => {
            Err(IpcError::invalid(format!("The snapshot could not be read ({})", messages::failure(error.reason))))
        }
    })
    .await
}

/// Restores a snapshot as a new revision of the catalog (the window asked first), then scans again.
#[tauri::command]
#[specta::specta]
pub fn health_restore_snapshot(ctx: AppState<'_>, snapshot_id: String) -> Result<(), IpcError> {
    const LABEL: &str = "Restoring a catalog snapshot";
    let id = snapshot(&snapshot_id)?;
    let app = Arc::clone(&ctx);
    let started = ctx.session.perform(LABEL, Lane::Library, false, move |operation| {
        let services = app.require_services().map_err(|error| error.message)?;
        let outcome = services.sync.restore_catalog_snapshot.run(&id, true);
        let restored = matches!(outcome, dms_application::common::OperationOutcome::Done { .. });
        operation.session().report(messages::of(&outcome, "Restored the catalog snapshot"));
        if restored { app.rescan(operation) } else { Ok(()) }
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, LABEL)) }
}

fn snapshot(id: &str) -> Result<CatalogSnapshotId, IpcError> {
    CatalogSnapshotId::new(id).map_err(|error| IpcError::invalid(error.to_string()))
}
