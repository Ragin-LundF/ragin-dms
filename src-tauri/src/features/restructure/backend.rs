//! The use cases the restructure screen calls (`AppServices`).

use dms_application::common::{OperationOutcome, Progress};
use dms_application::restructure::{PlanOutcome, RestructureHistoryOutcome, RestructureOutcome, RestructureScope};
use dms_domain::library::RelativePath;
use dms_domain::restructure::{FileNaming, RestructurePlan};
use dms_domain::settings::CatalogSettings;

use crate::context::AppContext;
use crate::dto::IpcError;

pub fn plan_restructure(
    ctx: &AppContext,
    scope: &RestructureScope,
    template: &str,
    naming: FileNaming,
) -> Result<PlanOutcome, IpcError> {
    Ok(ctx.require_services()?.organize.plan_restructure.run(scope, template, naming))
}

pub fn list_history(ctx: &AppContext) -> Result<RestructureHistoryOutcome, IpcError> {
    Ok(ctx.require_services()?.organize.list_restructure_history.run())
}

pub fn apply_plan(
    ctx: &AppContext,
    plan: &RestructurePlan,
    progress: &dyn Progress<(usize, usize)>,
) -> Result<RestructureOutcome, IpcError> {
    Ok(ctx.require_services()?.organize.apply_restructure_plan.run(plan, 0, progress))
}

pub fn reverse(
    ctx: &AppContext,
    name: &str,
    progress: &dyn Progress<(usize, usize)>,
) -> Result<RestructureOutcome, IpcError> {
    Ok(ctx.require_services()?.organize.reverse_restructure.run(name, progress))
}

pub fn remove_empty_folders(ctx: &AppContext, folders: &[RelativePath]) -> Result<OperationOutcome, IpcError> {
    Ok(ctx.require_services()?.organize.remove_empty_folders.run(folders, true))
}

pub fn save_path_template(ctx: &AppContext, template: &str) -> Result<OperationOutcome, IpcError> {
    let services = ctx.require_services()?;
    Ok(services.documents.change_settings.run(&format!("Path template {template}"), |settings| {
        let mut changed: CatalogSettings = settings.clone();
        changed.layout.path_template = Some(template.to_owned());
        changed
    }))
}
