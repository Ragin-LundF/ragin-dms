//! Commands of the quick look (Kotlin `QuickLookViewModel`): what it shows, values edited in place, open/show,
//! rename, move, trash and the AI runs for one document.

use dms_application::catalog::CatalogEdit;
use dms_domain::taxonomy::{CategoryId, TagName};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use super::wiring::{self, AiRun};
use crate::commands::AppState;
use crate::commands::session::file_manager_name;
use crate::dto::IpcError;
use crate::features::document::dto::{QuickFact, QuickLook, TrashQuestion};
use crate::features::document::view::edits::{self, DocumentEdit, FactPlan};
use crate::features::document::view::quick_look::{QuickLookContext, present_quick_look};

/// The quick look of `document_id`; `None` when it is no longer in the catalog.
#[tauri::command]
#[specta::specta]
pub fn quick_look(ctx: AppState<'_>, document_id: String) -> Result<Option<QuickLook>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let Some(document) = catalog.documents().get(&wiring::document_id(&document_id)?) else {
        return Ok(None);
    };
    let suggestion = wiring::pending_suggestion(&ctx, &document.id);
    let context = QuickLookContext {
        suggestion: suggestion.as_ref(),
        primary_folder_id: wiring::primary_folder(&ctx).map(|folder| folder.id),
        accept_file_names: wiring::accept_file_names(&ctx),
        today: wiring::today(&ctx),
    };
    Ok(Some(present_quick_look(&catalog, document, &context)))
}

/// Saves what was typed in place for `fact`; `false` keeps the edit open because the value was refused (the
/// message says why).
#[tauri::command]
#[specta::specta]
pub fn save_quick_fact(
    ctx: AppState<'_>,
    document_id: String,
    fact: QuickFact,
    typed: String,
) -> Result<bool, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    match edits::plan_fact(&catalog, document, fact, &typed) {
        FactPlan::Save(edit) => wiring::save_edit(&ctx, &edit).map(|()| true),
        FactPlan::Nothing => Ok(true),
        FactPlan::Refused(message) => {
            if let Some(message) = message {
                wiring::report_error(&ctx, message);
            }
            Ok(false)
        }
    }
}

/// Assigns `category_id`, or no category.
#[tauri::command]
#[specta::specta]
pub fn assign_document_category(
    ctx: AppState<'_>,
    document_id: String,
    category_id: Option<String>,
) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let category_id =
        category_id.map(|id| CategoryId::new(id).map_err(|error| IpcError::invalid(error.to_string()))).transpose()?;
    let edit = CatalogEdit::AssignCategory { document_id: document.id.clone(), category_id };
    wiring::save_edit(&ctx, &DocumentEdit::Catalog(edit))
}

#[tauri::command]
#[specta::specta]
pub fn remove_document_tag(ctx: AppState<'_>, document_id: String, tag: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let tag = TagName::of(&tag).map_err(|error| IpcError::invalid(error.to_string()))?;
    let edit = CatalogEdit::RemoveTags { document_id: document.id.clone(), tags: std::iter::once(tag).collect() };
    wiring::save_edit(&ctx, &DocumentEdit::Catalog(edit))
}

/// Opens the document's file in the primary folder with its default application.
#[tauri::command]
#[specta::specta]
pub fn open_document_file(app: AppHandle, ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let Some(file) = wiring::primary_file(&ctx, document) else {
        return Ok(());
    };
    if !file.is_file() || app.opener().open_path(file.to_string_lossy(), None::<&str>).is_err() {
        wiring::report_error(&ctx, format!("No application could open {}", document.path.file_name()));
    }
    Ok(())
}

/// Shows the document's file in the primary folder selected in the file manager.
#[tauri::command]
#[specta::specta]
pub fn show_document_in_file_manager(app: AppHandle, ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let Some(file) = wiring::primary_file(&ctx, document) else {
        return Ok(());
    };
    if app.opener().reveal_item_in_dir(file).is_err() {
        wiring::report_error(&ctx, format!("Could not open the {}", file_manager_name()));
    }
    Ok(())
}

/// Renames the file in every library folder to the typed `name` (extension included).
#[tauri::command]
#[specta::specta]
pub fn rename_document(ctx: AppState<'_>, document_id: String, name: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    match edits::rename_target(document, &name) {
        Ok(target) => wiring::move_document(&ctx, document, &target),
        Err(problem) => {
            wiring::report_error(&ctx, problem);
            Ok(())
        }
    }
}

/// Moves the file to the typed `folder` (`/` for the top level), keeping its name.
#[tauri::command]
#[specta::specta]
pub fn move_document_to_folder(ctx: AppState<'_>, document_id: String, folder: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    match edits::move_target(document, &folder) {
        Ok(target) => wiring::move_document(&ctx, document, &target),
        Err(problem) => {
            wiring::report_error(&ctx, problem);
            Ok(())
        }
    }
}

/// The question before "Move to trash…": every library folder the file leaves.
#[tauri::command]
#[specta::specta]
pub fn trash_question(ctx: AppState<'_>, document_id: String) -> Result<TrashQuestion, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let path = document.path.value();
    Ok(TrashQuestion {
        title: "Move to trash".to_owned(),
        message: "The file goes to the trash in every library folder and leaves the catalog.".to_owned(),
        affected: wiring::folder_names(&ctx)?.into_iter().map(|name| format!("{name}: {path}")).collect(),
        confirm_label: "Move to trash".to_owned(),
    })
}

/// Moves the file to the system trash in every library folder, after the user confirmed `trash_question`.
#[tauri::command]
#[specta::specta]
pub fn move_document_to_trash(ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    wiring::trash_document(&ctx, document)
}

/// "Classify with AI": the suggestion opens for review right away.
#[tauri::command]
#[specta::specta]
pub fn classify_document_with_ai(app: AppHandle, ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let label = format!("Classifying {} with AI", document.path.file_name());
    wiring::suggest_with_ai(&ctx, app, document, AiRun::Classify, &label)
}

/// "Tag with AI": new tags open for review right away; none are reported.
#[tauri::command]
#[specta::specta]
pub fn tag_document_with_ai(app: AppHandle, ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let label = format!("Tagging {} with AI", document.path.file_name());
    wiring::suggest_with_ai(&ctx, app, document, AiRun::Tag, &label)
}
