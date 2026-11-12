//! The command of "Turn into filters": the AI reads the sentence in the AI lane; the window adds the filters.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::{Arc, mpsc};

use dms_application::search::TranslateOutcome;

use super::dto::PlainSearchResultDto;
use super::view;
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::messages;
use crate::session::Lane;
use crate::session::reporting::{blocking, busy};

/// The library filters `sentence` stands for. Nothing is searched or changed.
#[tauri::command]
#[specta::specta]
pub async fn plain_search_translate(ctx: AppState<'_>, sentence: String) -> Result<PlainSearchResultDto, IpcError> {
    let services = Arc::clone(ctx.require_services()?);
    let label = "Turning your words into filters";
    let (sender, receiver) = mpsc::channel();
    let started = ctx.session.perform(label, Lane::Ai, false, move |_| {
        // The window may have stopped waiting; the filters are then not needed.
        let _ = sender.send(services.ai.translate_query.run(&sentence));
        Ok(())
    });
    if !started {
        return Err(busy(&ctx.session, Lane::Ai, label));
    }
    let outcome = blocking(move || receiver.recv().map_err(IpcError::internal)).await?;
    if let Some(problem) = messages::translate_problem(&outcome) {
        return Err(IpcError::invalid(problem));
    }
    match outcome {
        TranslateOutcome::Translated(translation) => Ok(view::result_dto(&translation)),
        TranslateOutcome::LibraryNotOpen | TranslateOutcome::BlankSentence | TranslateOutcome::Failed { .. } => {
            Err(IpcError::internal("the filters went missing"))
        }
    }
}
