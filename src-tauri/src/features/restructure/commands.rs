//! Commands of the restructure screen (Kotlin `RestructureViewModel`'s actions).
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;

use dms_application::common::Progress;
use dms_application::restructure::{PlanOutcome, RestructureHistoryOutcome, RestructureOutcome};
use dms_domain::library::RelativePath;
use dms_domain::restructure::{FileNaming, PathTemplate, TemplateParse};

use super::dto::{ApplyPreviewDto, FileNamingDto, PreviewDto, RestructureScopeDto, RestructureStateDto};
use super::{backend, state, view};
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::messages;
use crate::session::reporting::{Reporting, blocking, busy};
use crate::session::{Lane, OperationProgress};

#[tauri::command]
#[specta::specta]
pub fn restructure_state(ctx: AppState<'_>) -> Result<RestructureStateDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let saved = catalog.settings().layout.path_template.clone();
    Ok(RestructureStateDto {
        scopes: view::scope_options(&catalog),
        initial_template: saved.clone().unwrap_or_else(|| view::DEFAULT_TEMPLATE.to_owned()),
        saved_template: saved,
        placeholders: view::placeholders(),
        plan: state::plan().map(|plan| view::present_plan(&plan)),
        empty_folders: state::empty_folders(),
    })
}

/// The applied restructures that can be reversed, newest first.
#[tauri::command]
#[specta::specta]
pub async fn restructure_history(ctx: AppState<'_>) -> Result<Vec<String>, IpcError> {
    let ctx = Arc::clone(&ctx);
    blocking(move || match backend::list_history(&ctx)? {
        RestructureHistoryOutcome::Listed(names) => Ok(names),
        RestructureHistoryOutcome::LibraryNotOpen => Ok(Vec::new()),
    })
    .await
}

/// Plans the restructure of `scope` by `template`; nothing changes. The plan is kept for Apply.
#[tauri::command]
#[specta::specta]
pub async fn preview_restructure(
    ctx: AppState<'_>,
    scope: RestructureScopeDto,
    template: String,
    naming: FileNamingDto,
) -> Result<PreviewDto, IpcError> {
    if let TemplateParse::Invalid(problem) = PathTemplate::parse(&template) {
        return Ok(problem_dto(view::problem_text(problem)));
    }
    let Some(catalog) = ctx.catalog() else {
        return Ok(problem_dto("No library is open"));
    };
    let scope = view::resolve_scope(&catalog, &scope)?;
    let naming = match naming {
        FileNamingDto::Keep => FileNaming::Keep,
        FileNamingDto::DateAndTitle => FileNaming::DateAndTitle,
    };
    let ctx = Arc::clone(&ctx);
    let outcome = blocking(move || backend::plan_restructure(&ctx, &scope, &template, naming)).await?;
    Ok(match outcome {
        PlanOutcome::Planned(plan) => {
            let dto = view::present_plan(&plan);
            state::set_plan(Some(plan));
            PreviewDto::Planned { plan: dto }
        }
        PlanOutcome::InvalidTemplate(problem) => problem_dto(view::problem_text(problem)),
        PlanOutcome::LibraryNotOpen => problem_dto("No library is open"),
    })
}

/// What Apply would move with only the ticked target folders, for the confirmation.
#[tauri::command]
#[specta::specta]
pub fn restructure_apply_preview(included_folders: Vec<String>) -> Result<ApplyPreviewDto, IpcError> {
    let plan = state::plan().ok_or_else(no_plan)?;
    Ok(view::apply_preview(&view::chosen_plan(&plan, &included_folders)))
}

/// Applies the previewed plan's moves into the ticked target folders, as one cancellable operation.
#[tauri::command]
#[specta::specta]
pub fn apply_restructure(ctx: AppState<'_>, included_folders: Vec<String>) -> Result<(), IpcError> {
    let plan = state::plan().ok_or_else(no_plan)?;
    let chosen = view::chosen_plan(&plan, &included_folders);
    if chosen.moves.is_empty() {
        return Ok(());
    }
    let label = format!("Restructuring {} documents", chosen.moves.len());
    run_restructure(&ctx, &label, move |ctx, progress| backend::apply_plan(ctx, &chosen, progress))
}

/// Moves back every document still where the restructure `name` put it.
#[tauri::command]
#[specta::specta]
pub fn reverse_restructure(ctx: AppState<'_>, name: String) -> Result<(), IpcError> {
    let label = format!("Reversing restructure {name}");
    run_restructure(&ctx, &label, move |ctx, progress| backend::reverse(ctx, &name, progress))
}

/// Moves the folders an applied restructure left empty to the trash in every library folder.
#[tauri::command]
#[specta::specta]
pub fn remove_empty_folders(ctx: AppState<'_>, folders: Vec<String>) -> Result<(), IpcError> {
    state::set_empty_folders(Vec::new());
    let folders = folders
        .iter()
        .map(|folder| RelativePath::of(folder).map_err(|error| IpcError::invalid(error.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    let label = format!("Removing {} empty folders", folders.len());
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(&label, Lane::Library, false, move |op| {
        let outcome = backend::remove_empty_folders(&context, &folders).map_err(|error| error.message)?;
        op.session().report(messages::of(&outcome, "Removed empty folders"));
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, &label)) }
}

/// The user kept the empty folders.
#[tauri::command]
#[specta::specta]
pub fn dismiss_empty_folders() {
    state::set_empty_folders(Vec::new());
}

/// Remembers `template` as the library's default; inbox filing pre-fills its folder from it.
#[tauri::command]
#[specta::specta]
pub fn save_path_template(ctx: AppState<'_>, template: String) -> Result<(), IpcError> {
    let template = dms_domain::text::trim(&template).to_owned();
    let label = "Saving the path template";
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(label, Lane::Library, false, move |op| {
        let outcome = backend::save_path_template(&context, &template).map_err(|error| error.message)?;
        op.session().report(messages::of(&outcome, &format!("Saved {template} as the default template")));
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, label)) }
}

fn run_restructure(
    ctx: &Arc<AppContext>,
    label: &str,
    action: impl FnOnce(&AppContext, &dyn Progress<(usize, usize)>) -> Result<RestructureOutcome, IpcError> + Send + 'static,
) -> Result<(), IpcError> {
    let context = Arc::clone(ctx);
    let step = label.to_owned();
    let started = ctx.session.perform(label, Lane::Library, true, move |op| {
        let progress = Reporting::new(op, |&(done, total): &(usize, usize)| {
            op.progress(OperationProgress::new(&step, done, Some(total)));
        });
        let outcome = action(&context, &progress).map_err(|error| error.message)?;
        // Kotlin's cancelled coroutine reported nothing further; the session says "cancelled".
        if matches!(outcome, RestructureOutcome::Applied { cancelled: true, .. }) {
            return Ok(());
        }
        let report = view::report_of(&outcome);
        op.session().report(report.message);
        report.log_errors.into_iter().for_each(|line| op.session().error(line));
        state::set_plan(None);
        state::set_empty_folders(report.empty_folders);
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Library, label)) }
}

fn problem_dto(text: &str) -> PreviewDto {
    PreviewDto::Problem { text: text.to_owned() }
}

fn no_plan() -> IpcError {
    IpcError::invalid("Preview the plan first")
}
