//! Commands of the Ask screen (Kotlin `AskViewModel`'s actions). Both run in the AI lane, beside library work.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::ask::IndexProgress;

use super::dto::AskStateDto;
use super::{backend, state, view};
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::messages;
use crate::session::reporting::{Reporting, busy};
use crate::session::{Lane, OperationProgress};

#[tauri::command]
#[specta::specta]
pub fn ask_state(ctx: AppState<'_>) -> AskStateDto {
    AskStateDto { entries: state::entries(), scopes: view::scope_labels(ctx.catalog().as_ref()) }
}

/// Asks `question` in the scope at `scope_index` of [`AskStateDto::scopes`]; the answer arrives in the entries.
#[tauri::command]
#[specta::specta]
pub fn ask_library(ctx: AppState<'_>, question: String, scope_index: u32) {
    let question = dms_domain::text::trim(&question).to_owned();
    if question.is_empty() {
        return;
    }
    let catalog = ctx.catalog();
    let scope = view::scope_query(catalog.as_ref(), usize::try_from(scope_index).unwrap_or(usize::MAX));
    let position = state::push(view::asking(&question));
    let label = "Asking the library";
    let context = Arc::clone(&ctx);
    let asked = question.clone();
    let started = ctx.session.perform(label, Lane::Ai, true, move |op| {
        let question = asked.clone();
        let result = op.run_until_cancelled(move || backend::ask_library(&context, &question, &scope));
        if let Some(Ok(outcome)) = &result
            && !op.is_cancelled()
        {
            state::replace(position, view::entry_of(&asked, outcome));
        }
        state::stop_waiting();
        // A cancelled question (`None`) ends quietly; its answer is dropped when it arrives.
        result.map_or(Ok(()), |answer| answer.map(|_| ()).map_err(|error| error.message))
    });
    if !started {
        let running = ctx.session.state().busy_in(Lane::Ai).unwrap_or(label).to_owned();
        state::replace(position, view::noted(&question, Some(messages::wait_for(&running))));
    }
}

/// Embeds what is new or changed since the last run; the rest of the index stays.
#[tauri::command]
#[specta::specta]
pub fn update_ask_index(ctx: AppState<'_>) -> Result<(), IpcError> {
    let label = "Updating the index";
    let context = Arc::clone(&ctx);
    let started = ctx.session.perform(label, Lane::Ai, true, move |op| {
        let indexed = AtomicUsize::new(0);
        let progress = Reporting::new(op, |step: &IndexProgress| {
            op.progress(OperationProgress::new(label, step.done, Some(step.total)));
            if step.indexed {
                indexed.fetch_add(1, Ordering::Relaxed);
            }
        });
        let last = backend::index_library(&context, &progress).map_err(|error| error.message)?;
        if op.is_cancelled() {
            return Ok(());
        }
        let failure = last.and_then(|step| step.failure);
        op.session().report(view::index_message(indexed.load(Ordering::Relaxed), failure.as_deref()));
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(&ctx.session, Lane::Ai, label)) }
}
