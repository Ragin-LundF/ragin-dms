//! The commands of the emergency screen, of the quick look row and of the "Add to emergency folder" button. Every
//! change runs in the library lane; the ZIP password is passed to the use case only, never logged.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use dms_application::emergency::{EmergencyOutcome, OverviewTarget};
use dms_domain::contract::ContractId;
use dms_domain::document::DocumentId;
use dms_domain::emergency::EmergencyEntry;
use dms_domain::library::RelativePath;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use super::dto::{
    EmergencyChoiceDto, EmergencyEditDto, EmergencyMembershipDto, EmergencyOverviewTargetDto, EmergencyStateDto,
};
use super::view;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::letters::commands::in_lane;
use crate::messages;
use crate::session::{Lane, UiMessage};

#[tauri::command]
#[specta::specta]
pub fn emergency_state(ctx: AppState<'_>) -> Result<EmergencyStateDto, IpcError> {
    let today = ctx.require_services()?.library.clock.today();
    Ok(view::present_emergency(&ctx.require_catalog()?, today))
}

/// Adds, removes, includes or excludes an item, or changes the intro or the refresh period. `false` when refused
/// (the message says why).
#[tauri::command]
#[specta::specta]
pub async fn emergency_edit(ctx: AppState<'_>, edit: EmergencyEditDto) -> Result<bool, IpcError> {
    let edit = view::edit_of(&edit).map_err(IpcError::invalid)?;
    let outcome = in_lane(&ctx, "Saving the emergency folder", Lane::Library, move |services| {
        services.organize.edit_emergency_folder.run(&edit)
    })
    .await?;
    let done = matches!(outcome, dms_application::common::OperationOutcome::Done { .. });
    if !done {
        ctx.session.report(messages::of(&outcome, ""));
    }
    Ok(done)
}

#[tauri::command]
#[specta::specta]
pub async fn emergency_mark_refreshed(ctx: AppState<'_>) -> Result<(), IpcError> {
    let outcome = in_lane(&ctx, "Marking the emergency folder as refreshed", Lane::Library, |services| {
        services.organize.mark_emergency_folder_refreshed.run()
    })
    .await?;
    ctx.session.report(messages::of(&outcome, "Marked the emergency folder as refreshed"));
    Ok(())
}

/// Saves the overview PDF into a library folder or outside the library; with `open`, a file outside is opened in
/// its default application afterwards, for printing. `false` when nothing was saved.
#[tauri::command]
#[specta::specta]
pub async fn emergency_save_overview(
    app: AppHandle,
    ctx: AppState<'_>,
    target: EmergencyOverviewTargetDto,
    open: bool,
) -> Result<bool, IpcError> {
    let target = match target {
        EmergencyOverviewTargetDto::Library { folder } => OverviewTarget::Library {
            folder: RelativePath::of(&folder).map_err(|error| IpcError::invalid(error.to_string()))?,
        },
        EmergencyOverviewTargetDto::Outside { path } => {
            let (directory, name) = view::split_path(&path).ok_or_else(|| IpcError::invalid("Choose a file"))?;
            OverviewTarget::Outside { directory, name }
        }
    };
    let outside = matches!(target, OverviewTarget::Outside { .. });
    let outcome = in_lane(&ctx, "Saving the emergency overview", Lane::Library, move |services| {
        services.organize.create_emergency_overview.run(&target)
    })
    .await?;
    ctx.session.report(messages::emergency_outcome(&outcome, "Saved the emergency overview"));
    let EmergencyOutcome::Saved { location, .. } = outcome else { return Ok(false) };
    if open && outside && app.opener().open_path(&location, None::<&str>).is_err() {
        ctx.session.report(UiMessage::error(format!("No application could open {location}")));
    }
    Ok(true)
}

/// Writes the AES-256 encrypted ZIP to `path`, chosen in the save dialog outside the library. `false` when nothing
/// was written.
#[tauri::command]
#[specta::specta]
pub async fn emergency_export_zip(ctx: AppState<'_>, path: String, password: String) -> Result<bool, IpcError> {
    let (directory, name) = view::split_path(&path).ok_or_else(|| IpcError::invalid("Choose a file"))?;
    let name = name.strip_suffix(".zip").map(str::to_owned).unwrap_or(name);
    let outcome = in_lane(&ctx, "Writing the encrypted emergency folder", Lane::Library, move |services| {
        services.organize.export_emergency_folder.run(&directory, &name, &password)
    })
    .await?;
    ctx.session.report(messages::emergency_outcome(&outcome, "Wrote the encrypted emergency folder"));
    Ok(matches!(outcome, EmergencyOutcome::Saved { .. }))
}

/// Whether the document or the contract is in the emergency folder.
#[tauri::command]
#[specta::specta]
pub fn emergency_membership(
    ctx: AppState<'_>,
    document_id: Option<String>,
    contract_id: Option<String>,
) -> Result<EmergencyMembershipDto, IpcError> {
    let invalid = |error: dms_domain::DomainError| IpcError::invalid(error.to_string());
    let entry = match (document_id, contract_id) {
        (Some(id), _) => EmergencyEntry::Document(DocumentId::new(id).map_err(invalid)?),
        (None, Some(id)) => EmergencyEntry::Contract(ContractId::new(id).map_err(invalid)?),
        (None, None) => return Err(IpcError::invalid("A document or a contract is needed")),
    };
    Ok(view::membership(&ctx.require_catalog()?, &entry))
}

#[tauri::command]
#[specta::specta]
pub fn emergency_document_candidates(ctx: AppState<'_>, query: String) -> Result<Vec<EmergencyChoiceDto>, IpcError> {
    Ok(view::document_candidates(&ctx.require_catalog()?, &query))
}
