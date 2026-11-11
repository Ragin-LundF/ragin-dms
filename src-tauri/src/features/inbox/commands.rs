//! The Inbox screen's commands (Kotlin `InboxViewModel`).
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use dms_application::common::OperationOutcome;
use dms_domain::catalog::Catalog;
use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;
use dms_domain::task::OpenTask;
use dms_domain::text;

use super::dto::InboxStateDto;
use super::view::{present_inbox, recent_targets, remember_target};
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::document::view::texts::suggest_message;
use crate::features::settings::run::{perform, services, try_perform};
use crate::messages;
use crate::session::Lane;

/// The inbox of the open library; switched off while none is open.
#[tauri::command]
#[specta::specta]
pub fn inbox_state(ctx: AppState<'_>) -> InboxStateDto {
    let catalog = ctx.catalog();
    let suggestions: Vec<_> = ctx
        .dashboard()
        .map(|dashboard| dashboard.tasks)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|task| match task {
            OpenTask::ReviewSuggestion(suggestion) => Some(suggestion),
            _ => None,
        })
        .collect();
    present_inbox(catalog.as_ref(), &suggestions, ctx.session.state().ai_enabled, recent_targets())
}

/// Lets the AI suggest a classification for one inbox document, as "Analyzing {file}" in the AI lane.
#[tauri::command]
#[specta::specta]
pub fn analyze_inbox_document(ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let id = parse_document_id(document_id)?;
    let name = name_of(&catalog, &id);
    perform(&ctx, &format!("Analyzing {name}"), Lane::Ai, move |app, operation| {
        let outcome = services(app)?.ai.suggest_classification.run(&id);
        operation.session().report(suggest_message(&outcome, &name));
        operation.session().changed();
        Ok(())
    })
}

/// "Analyze all": classifies the inbox documents without a classification or suggestion; cancellable.
#[tauri::command]
#[specta::specta]
pub fn analyze_inbox(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.analyze_inbox()
}

/// Files an inbox document into `folder` (created where missing), renamed to `file_name` when that differs, as
/// "Filing {file}". `false` when the library lane is busy, so the picker stays open.
#[tauri::command]
#[specta::specta]
pub fn file_inbox_document(
    ctx: AppState<'_>,
    document_id: String,
    folder: String,
    file_name: String,
) -> Result<bool, IpcError> {
    let catalog = ctx.require_catalog()?;
    let id = parse_document_id(document_id)?;
    let segments: Vec<&str> = folder.split('/').map(text::trim).filter(|segment| !segment.is_empty()).collect();
    let folder = RelativePath::from_segments(&segments).map_err(|error| IpcError::invalid(error.to_string()))?;
    let name = name_of(&catalog, &id);
    let new_name = Some(text::trim(&file_name)).filter(|typed| !typed.is_empty() && *typed != name).map(str::to_owned);
    Ok(try_perform(&ctx, &format!("Filing {name}"), Lane::Library, move |app, operation| {
        let outcome = services(app)?.organize.file_inbox_document.run(&id, &folder, new_name.as_deref());
        operation.session().report(messages::of(&outcome, &format!("Filed {name} in {folder}")));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            remember_target(folder.value());
        }
        Ok(())
    }))
}

fn name_of(catalog: &Catalog, id: &DocumentId) -> String {
    catalog.documents().get(id).map_or_else(|| id.value().to_owned(), |document| document.path.file_name().to_owned())
}

fn parse_document_id(value: String) -> Result<DocumentId, IpcError> {
    DocumentId::new(value).map_err(|error| IpcError::invalid(error.to_string()))
}
