//! Commands of the folded details (Kotlin `DetailsController`): fields, note, links, rule and retention.

use dms_application::document::DetailEdit;
use dms_domain::taxonomy::CategoryId;

use super::wiring;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::document::dto::{LinkedDocument, TypedField};
use crate::features::document::view::details::{candidate_of, link_candidates};
use crate::features::document::view::edits::{self, DocumentEdit};

/// Creates the standard fields (amount, due date, cancel by, contract end, warranty) the library lacks.
#[tauri::command]
#[specta::specta]
pub fn add_standard_fields(ctx: AppState<'_>) -> Result<(), IpcError> {
    wiring::change_settings(
        &ctx,
        "Adding the standard fields",
        "Standard fields",
        "Added the standard fields",
        edits::settings_with_standard_fields,
    )
}

/// Saves all typed field values together; one that does not fit saves nothing and is reported.
#[tauri::command]
#[specta::specta]
pub fn save_document_fields(ctx: AppState<'_>, document_id: String, fields: Vec<TypedField>) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    match edits::plan_fields(&catalog, document, &fields) {
        Ok(edit) => wiring::save_edit(&ctx, &DocumentEdit::Details(edit)),
        Err(problem) => {
            wiring::report_error(&ctx, problem);
            Ok(())
        }
    }
}

/// Saves the note when the field loses the focus; an unchanged note saves nothing.
#[tauri::command]
#[specta::specta]
pub fn save_document_note(ctx: AppState<'_>, document_id: String, note: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    match edits::plan_note(document, &note) {
        Some(edit) => wiring::save_edit(&ctx, &DocumentEdit::Details(edit)),
        None => Ok(()),
    }
}

/// Sets the keep-until date (`YYYY-MM-DD`), or removes it when empty.
#[tauri::command]
#[specta::specta]
pub fn save_keep_until(ctx: AppState<'_>, document_id: String, typed: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    save_or_report(&ctx, edits::plan_keep_until(document, &typed))
}

/// Sets how many years documents of `category_id` are kept; empty removes the period.
#[tauri::command]
#[specta::specta]
pub fn save_category_years(ctx: AppState<'_>, category_id: String, typed: String) -> Result<(), IpcError> {
    let category_id = CategoryId::new(category_id).map_err(|error| IpcError::invalid(error.to_string()))?;
    save_or_report(&ctx, edits::plan_category_years(category_id, &typed))
}

/// A manual rule from the document: its correspondent's documents get its category and tags proposed.
#[tauri::command]
#[specta::specta]
pub fn create_rule_from_document(ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let rule = match edits::rule_from(document, wiring::new_rule_id()) {
        Ok(rule) => rule,
        Err(problem) => {
            wiring::report_error(&ctx, problem);
            return Ok(());
        }
    };
    let correspondent = document.metadata.correspondent.clone().unwrap_or_default();
    let label = format!("Creating {}", rule.name);
    let done = format!("Created {}", rule.name);
    let name = rule.name.clone();
    wiring::change_settings(&ctx, &label, &name, &done, move |settings| {
        edits::settings_with_rule(settings, &rule, &correspondent)
    })
}

/// Up to 30 documents to link `document_id` to, matching `query`.
#[tauri::command]
#[specta::specta]
pub fn link_candidates_of(
    ctx: AppState<'_>,
    document_id: String,
    query: String,
) -> Result<Vec<LinkedDocument>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    Ok(link_candidates(&catalog, document, &query))
}

/// Links two documents; a blank label is none.
#[tauri::command]
#[specta::specta]
pub fn link_documents(ctx: AppState<'_>, document_id: String, other_id: String, label: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let other = wiring::find_document(&catalog, &other_id)?;
    let label = Some(dms_domain::text::trim(&label).to_owned()).filter(|label| !label.is_empty());
    let edit = DetailEdit::Link { first: document.id.clone(), second: other.id.clone(), label };
    wiring::save_edit(&ctx, &DocumentEdit::Details(edit))
}

#[tauri::command]
#[specta::specta]
pub fn unlink_documents(ctx: AppState<'_>, document_id: String, other_id: String) -> Result<(), IpcError> {
    let first = wiring::document_id(&document_id)?;
    let second = wiring::document_id(&other_id)?;
    wiring::save_edit(&ctx, &DocumentEdit::Details(DetailEdit::Unlink { first, second }))
}

/// The documents closest in meaning, from the Ask index; empty while it has none.
#[tauri::command]
#[specta::specta]
pub fn related_documents_of(ctx: AppState<'_>, document_id: String) -> Result<Vec<LinkedDocument>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let related = wiring::related(&ctx, &wiring::document_id(&document_id)?)?;
    Ok(related.iter().filter_map(|id| catalog.documents().get(id)).map(candidate_of).collect())
}

fn save_or_report(ctx: &crate::context::AppContext, plan: Result<DetailEdit, &'static str>) -> Result<(), IpcError> {
    match plan {
        Ok(edit) => wiring::save_edit(ctx, &DocumentEdit::Details(edit)),
        Err(problem) => {
            wiring::report_error(ctx, problem);
            Ok(())
        }
    }
}
