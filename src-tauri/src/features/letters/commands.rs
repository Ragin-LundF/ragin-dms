//! Commands of the letter dialog: drafting runs in the AI lane when the AI writes, saving in the library lane.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::{Arc, mpsc};

use dms_application::letter::{DraftLetterInput, SaveLetterInput, SaveLetterOutcome};
use dms_application::services::AppServices;
use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;
use dms_domain::text;

use super::dto::{LetterDraftDto, LetterDraftInputDto, LetterSaveInputDto, LetterSavedDto};
use super::view;
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::messages;
use crate::session::Lane;
use crate::session::reporting::{blocking, busy};

/// A letter draft for the dialog: by the AI when it is on, otherwise (or when it fails) from the template.
#[tauri::command]
#[specta::specta]
pub async fn letter_draft(ctx: AppState<'_>, input: LetterDraftInputDto) -> Result<LetterDraftDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let sender = view::letter_owner(&catalog);
    let draft = DraftLetterInput {
        kind: view::kind_of(input.kind),
        document_ids: parse_ids(&input.document_ids)?,
        recipient: input.recipient.filter(|name| !text::is_blank(name)),
        subject_facts: input.subject_facts,
        sender: sender.clone(),
    };
    let ctx = Arc::clone(&ctx);
    let outcome = if ctx.session.state().ai_enabled {
        in_lane(&ctx, "Drafting a letter", Lane::Ai, move |services| services.documents.draft_letter.run(&draft))
            .await?
    } else {
        let services = Arc::clone(ctx.require_services()?);
        blocking(move || Ok(services.documents.draft_letter.run(&draft))).await?
    };
    if let Some(problem) = messages::letter_problem(&outcome) {
        return Err(IpcError::invalid(problem));
    }
    view::draft_dto(outcome, sender, &catalog).ok_or_else(|| IpcError::internal("the draft went missing"))
}

/// Saves the edited letter as a PDF in `folder` of every library folder and catalogs it.
#[tauri::command]
#[specta::specta]
pub async fn letter_save(ctx: AppState<'_>, input: LetterSaveInputDto) -> Result<LetterSavedDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let letter = SaveLetterInput {
        document_ids: parse_ids(&input.document_ids)?,
        folder: RelativePath::of(&input.folder).map_err(|error| IpcError::invalid(error.to_string()))?,
        sender: view::letter_owner(&catalog),
        recipient: view::party_of(&input.recipient),
        subject: input.subject,
        body: input.body,
        language: view::language_of(input.language),
    };
    let ctx = Arc::clone(&ctx);
    let outcome =
        in_lane(&ctx, "Saving the letter", Lane::Library, move |services| services.documents.save_letter.run(&letter))
            .await?;
    let message = messages::letter_saved(&outcome);
    match outcome {
        SaveLetterOutcome::Saved { document_id, path, .. } => {
            ctx.session.report(message);
            Ok(LetterSavedDto { document_id: document_id.value().to_owned(), path: path.value().to_owned() })
        }
        SaveLetterOutcome::Refused(_) => Err(IpcError::invalid(message.text)),
    }
}

/// Runs `job` as the operation `label` of `lane` and waits for its result; `busy` while the lane runs another one.
pub(crate) async fn in_lane<T: Send + 'static>(
    ctx: &Arc<AppContext>,
    label: &str,
    lane: Lane,
    job: impl FnOnce(&AppServices) -> T + Send + 'static,
) -> Result<T, IpcError> {
    let services = Arc::clone(ctx.require_services()?);
    let (sender, receiver) = mpsc::channel();
    let started = ctx.session.perform(label, lane, false, move |_| {
        // The window may have stopped waiting; the result is then not needed.
        let _ = sender.send(job(&services));
        Ok(())
    });
    if !started {
        return Err(busy(&ctx.session, lane, label));
    }
    blocking(move || receiver.recv().map_err(IpcError::internal)).await
}

fn parse_ids(raw: &[String]) -> Result<Vec<DocumentId>, IpcError> {
    raw.iter().map(|id| DocumentId::new(id).map_err(|error| IpcError::invalid(error.to_string()))).collect()
}
