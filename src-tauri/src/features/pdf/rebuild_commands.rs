//! Commands of "Convert to real PDF" (Kotlin `RebuildController`): the run in the AI lane, the comparison and the
//! choice of what to keep.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;

use dms_application::pdf::rebuild::{RebuildChoice, RebuildOutcome};

use super::commands::document;
use super::dto::{RebuildChoiceDto, RebuildCompareDto};
use super::state::{self, Comparison};
use super::{backend, rebuild_view};
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::messages;
use crate::session::reporting::{Reporting, busy};
use crate::session::{Lane, UiMessage};

/// Converts the scan `document_id` into a PDF with real text; the comparison opens when it is done.
#[tauri::command]
#[specta::specta]
pub fn start_rebuild(ctx: AppState<'_>, document_id: String) -> Result<(), IpcError> {
    if state::comparison().is_some() {
        ctx.session.report(UiMessage::error(rebuild_view::OPEN_COMPARISON));
        return Ok(());
    }
    let document = document(&ctx.require_catalog()?, &document_id)?;
    let name = document.path.file_name().to_owned();
    let label = format!("Converting {name} to a real PDF");
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(&label, Lane::Ai, true, move |op| {
        let progress =
            Reporting::new(op, |&(page, pages): &(usize, usize)| op.progress(rebuild_view::progress_of(page, pages)));
        let outcome = backend::rebuild(&context, &document.id, &progress).map_err(|error| error.message)?;
        if let Some(message) = rebuild_view::rebuild_message(&outcome, &name) {
            op.session().report(message);
        }
        if let RebuildOutcome::Rebuilt { rebuilt, original_size } = outcome {
            state::set_comparison(Some(Comparison {
                document_id: document.id,
                path: document.path,
                rebuilt,
                original_size,
            }));
        }
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Ai, &label)) }
}

/// The open comparison, or `None`.
#[tauri::command]
#[specta::specta]
pub fn rebuild_compare() -> Option<RebuildCompareDto> {
    state::comparison().map(|comparison| rebuild_view::present_compare(&comparison))
}

/// Keeps the original, the rebuild (the scan goes to the trash; the screen asked first) or both.
#[tauri::command]
#[specta::specta]
pub fn choose_rebuild(ctx: AppState<'_>, choice: RebuildChoiceDto) {
    let Some(comparison) = state::comparison() else {
        return;
    };
    let choice = match choice {
        RebuildChoiceDto::KeepOriginal => RebuildChoice::KeepOriginal,
        RebuildChoiceDto::ReplaceWithRebuild => RebuildChoice::ReplaceWithRebuild,
        RebuildChoiceDto::KeepBoth => RebuildChoice::KeepBoth,
    };
    let name = comparison.path.file_name().to_owned();
    let label = rebuild_view::choice_label(choice, &name);
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(&label, Lane::Library, false, move |op| {
        let outcome = backend::keep_rebuild(&context, &comparison.document_id, &comparison.rebuilt, choice)
            .map_err(|error| error.message)?;
        op.session().report(rebuild_view::choice_message(&outcome, choice, &name));
        if rebuild_view::closes_comparison(&outcome) {
            state::set_comparison(None);
        }
        Ok(())
    });
    if !started {
        let running = ctx.session.state().busy_in(Lane::Library).unwrap_or(&label).to_owned();
        ctx.session.report(UiMessage::error(messages::wait_for(&running)));
    }
}
