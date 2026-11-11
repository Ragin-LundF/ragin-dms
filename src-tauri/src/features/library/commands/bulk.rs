//! The bulk bar's actions on the ticked documents (Kotlin `BulkActions`). Each returns `true` when the ticks should
//! clear: the operation started, or no document needed the change. `false` means nothing ran (a busy lane, or a
//! refused input that was reported), so the ticks stay.

use dms_application::catalog::CatalogEdit;
use dms_application::classification::SuggestOutcome;
use dms_application::common::OperationOutcome;
use dms_application::port::ai::ClassificationFailure;
use dms_domain::document::Document;
use dms_domain::library::RelativePath;
use dms_domain::taxonomy::TagName;

use super::documents_of;
use super::operation::{documents_text, run_each, start};
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::document::view::texts::suggest_message;
use crate::features::library::dto::params::{parse_category_id, parse_document_ids, parse_path};
use crate::session::{Lane, OperationProgress, UiMessage};

/// Files the documents under a category, or removes their category with `null`.
#[tauri::command]
#[specta::specta]
pub async fn library_bulk_assign_category(
    ctx: AppState<'_>,
    document_ids: Vec<String>,
    category_id: Option<String>,
) -> Result<bool, IpcError> {
    let ids = parse_document_ids(&document_ids)?;
    let category_id = category_id.as_deref().map(parse_category_id).transpose()?;
    let catalog = ctx.require_catalog()?;
    let changing: Vec<&Document> =
        documents_of(&catalog, &ids).into_iter().filter(|document| document.category_id != category_id).collect();
    if changing.is_empty() {
        return Ok(true);
    }
    let step = match category_id.as_ref().and_then(|id| catalog.categories().get(id)) {
        Some(category) => format!("Filing under {}", category.name),
        None => "Removing the category".to_owned(),
    };
    run_each(ctx.inner(), &changing, &step, move |services, document| {
        let edit = CatalogEdit::AssignCategory { document_id: document.id.clone(), category_id: category_id.clone() };
        services.documents.edit_catalog.run(&edit)
    })
}

/// Adds a tag, known or new, to the documents. A tag that cannot be one is reported: "A tag needs a name".
#[tauri::command]
#[specta::specta]
pub async fn library_bulk_add_tag(ctx: AppState<'_>, document_ids: Vec<String>, tag: String) -> Result<bool, IpcError> {
    let ids = parse_document_ids(&document_ids)?;
    let Some(tag) = TagName::parse(&tag) else {
        ctx.session.report(UiMessage::error("A tag needs a name"));
        return Ok(false);
    };
    let catalog = ctx.require_catalog()?;
    let changing: Vec<&Document> =
        documents_of(&catalog, &ids).into_iter().filter(|document| !document.tags.contains(&tag)).collect();
    if changing.is_empty() {
        return Ok(true);
    }
    let step = format!("Tagging with {}", tag.value());
    run_each(ctx.inner(), &changing, &step, move |services, document| {
        let mut tags = document.tags.clone();
        tags.clear();
        tags.insert(tag.clone());
        services.documents.edit_catalog.run(&CatalogEdit::AddTags { document_id: document.id.clone(), tags })
    })
}

/// Moves the documents into `target` in every library folder, keeping their names; one already there stays.
#[tauri::command]
#[specta::specta]
pub async fn library_bulk_move(ctx: AppState<'_>, document_ids: Vec<String>, target: String) -> Result<bool, IpcError> {
    let ids = parse_document_ids(&document_ids)?;
    let target = parse_path(&target)?;
    let catalog = ctx.require_catalog()?;
    let moving: Vec<&Document> = documents_of(&catalog, &ids)
        .into_iter()
        .filter(|document| document.path.parent().unwrap_or_else(RelativePath::root) != target)
        .collect();
    if moving.is_empty() {
        return Ok(true);
    }
    let step = format!("Moving to {}", if target.is_root() { "the top level" } else { target.value() });
    run_each(ctx.inner(), &moving, &step, move |services, document| match target.resolve(document.path.file_name()) {
        Ok(new_path) => services.documents.move_document.run(&document.id, &new_path),
        Err(error) => OperationOutcome::InvalidInput(error),
    })
}

/// Moves the documents to the system trash from every library folder; the screen asked first, naming each.
#[tauri::command]
#[specta::specta]
pub async fn library_bulk_trash(ctx: AppState<'_>, document_ids: Vec<String>) -> Result<bool, IpcError> {
    let ids = parse_document_ids(&document_ids)?;
    let catalog = ctx.require_catalog()?;
    let documents = documents_of(&catalog, &ids);
    if documents.is_empty() {
        return Ok(true);
    }
    run_each(ctx.inner(), &documents, "Moving to the trash", |services, document| {
        services.documents.trash_document.run(&document.id, true)
    })
}

/// Lets the AI suggest tags for every document in the AI lane; the suggestions wait under Needs review. Stops at
/// the first "provider unavailable"; other failures go to the log.
#[tauri::command]
#[specta::specta]
pub async fn library_bulk_tag_with_ai(ctx: AppState<'_>, document_ids: Vec<String>) -> Result<bool, IpcError> {
    let ids = parse_document_ids(&document_ids)?;
    let catalog = ctx.require_catalog()?;
    let documents: Vec<Document> = documents_of(&catalog, &ids).into_iter().cloned().collect();
    let label = format!("Tagging {} with AI", documents_text(documents.len()));
    let step = label.clone();
    let started = start(ctx.inner(), &label, Lane::Ai, true, move |services, operation| {
        let mut suggested = 0;
        for (index, document) in documents.iter().enumerate() {
            if operation.is_cancelled() {
                return;
            }
            operation.progress(
                OperationProgress::new(&step, index, Some(documents.len())).with_detail(document.path.value()),
            );
            let outcome = services.ai.suggest_tags.run(&document.id);
            match &outcome {
                SuggestOutcome::Suggested(suggestion) => {
                    if !suggestion.tags.is_empty() {
                        suggested += 1;
                        operation.session().changed();
                    }
                }
                SuggestOutcome::Failed { reason: ClassificationFailure::ProviderUnavailable, .. } => {
                    operation.session().report(suggest_message(&outcome, document.path.file_name()));
                    return;
                }
                _ => operation.session().error(suggest_message(&outcome, document.path.file_name()).text),
            }
        }
        operation.session().report(UiMessage::info(format!(
            "The AI suggested tags for {suggested} of {} document(s); review them under Needs review",
            documents.len()
        )));
    });
    match started {
        Ok(()) => Ok(true),
        Err(error) if error.code == "busy" => Ok(false),
        Err(error) => Err(error),
    }
}
