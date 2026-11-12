//! Commands of the review of an AI suggestion and of the quick look's suggestion card (Kotlin `ReviewController`,
//! `QuickLookViewModel.acceptSuggestion/dismissSuggestion`).

use super::wiring;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::features::cases::commands::apply_review_choice;
use crate::features::cases::dto::CasesReviewChoiceDto;
use crate::features::document::dto::{Review, ReviewRowDraft};
use crate::features::document::view::review::{approval_of, present_review};

/// The review of the suggestion that waits for `document_id`; `None` when there is none. A document that left the
/// catalog is reported as "The document of this suggestion is gone".
#[tauri::command]
#[specta::specta]
pub fn suggestion_review(ctx: AppState<'_>, document_id: String) -> Result<Option<Review>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let id = wiring::document_id(&document_id)?;
    let Some(document) = catalog.documents().get(&id) else {
        wiring::report_error(&ctx, "The document of this suggestion is gone");
        return Ok(None);
    };
    let Some(suggestion) = wiring::pending_suggestion(&ctx, &id) else {
        return Ok(None);
    };
    Ok(Some(present_review(&catalog, document, &suggestion, wiring::accept_file_names(&ctx))))
}

/// Applies the checked rows, then the case choice if any. Returns why the review stays open (a value that does not
/// fit, or a busy library), or `None` when the approval started and the review closes.
#[tauri::command]
#[specta::specta]
pub fn approve_review(
    ctx: AppState<'_>,
    document_id: String,
    rows: Vec<ReviewRowDraft>,
    cases: Option<CasesReviewChoiceDto>,
) -> Result<Option<String>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let approval = match approval_of(&catalog, document, &rows) {
        Ok(approval) => approval,
        Err(problem) => return Ok(Some(problem)),
    };
    let id = document.id.clone();
    let then = move |services: &_, catalog: &_| {
        cases.as_ref().map(|choice| apply_review_choice(services, catalog, &id, choice))
    };
    match wiring::approve(&ctx, document, approval, then) {
        Ok(()) => Ok(None),
        Err(error) if error.code == "busy" => Ok(Some(error.message)),
        Err(error) => Err(error),
    }
}

/// "Accept all" on the suggestion card: the suggestion approved unchanged. Returns the problem when the approval was
/// refused before anything changed; the window then opens the review with it.
#[tauri::command]
#[specta::specta]
pub fn accept_suggestion(ctx: AppState<'_>, document_id: String) -> Result<Option<String>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let Some(suggestion) = wiring::pending_suggestion(&ctx, &document.id) else {
        return Ok(None);
    };
    let review = present_review(&catalog, document, &suggestion, wiring::accept_file_names(&ctx));
    let rows: Vec<ReviewRowDraft> = review
        .rows
        .into_iter()
        .map(|row| ReviewRowDraft { key: row.key, accepted: row.accepted, value: row.value })
        .collect();
    approve_review(ctx, document_id, rows, None)
}

/// "Reject suggestion" in the review; `false` while the library is busy, so the review stays open.
#[tauri::command]
#[specta::specta]
pub fn reject_review(ctx: AppState<'_>, document_id: String) -> Result<bool, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let label = format!("Rejecting the suggestion for {}", document.path.value());
    started(wiring::reject(&ctx, document, &label))
}

/// "Dismiss" on the suggestion card; like "Reject" on the dashboard.
#[tauri::command]
#[specta::specta]
pub fn dismiss_suggestion(ctx: AppState<'_>, document_id: String) -> Result<bool, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    let label = format!("Rejecting the suggestion for {}", document.path.file_name());
    started(wiring::reject(&ctx, document, &label))
}

fn started(result: Result<(), IpcError>) -> Result<bool, IpcError> {
    match result {
        Ok(()) => Ok(true),
        Err(error) if error.code == "busy" => Ok(false),
        Err(error) => Err(error),
    }
}
