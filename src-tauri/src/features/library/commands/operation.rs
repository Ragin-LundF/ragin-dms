//! Running the library's changes as session operations (Kotlin `session.perform` in the library's controllers).

use std::sync::{Arc, mpsc};

use dms_application::common::OperationOutcome;
use dms_application::services::AppServices;
use dms_domain::document::Document;

use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::library::view::state::count;
use crate::session::reporting::{blocking, busy};
use crate::session::{Lane, Operation, UiMessage};

/// "1 document" or "N documents".
pub fn documents_text(count: usize) -> String {
    if count == 1 { "1 document".to_owned() } else { format!("{count} documents") }
}

/// Starts `job` as the operation `label`; `busy` when its lane runs another one.
pub fn start(
    ctx: &Arc<AppContext>,
    label: &str,
    lane: Lane,
    cancellable: bool,
    job: impl FnOnce(&AppServices, &Operation) + Send + 'static,
) -> Result<(), IpcError> {
    let services = Arc::clone(ctx.require_services()?);
    if ctx.session.perform(label, lane, cancellable, move |operation| {
        job(&services, operation);
        Ok(())
    }) {
        Ok(())
    } else {
        Err(busy(&ctx.session, lane, label))
    }
}

/// Runs `job` as a library operation and waits for its result, so the screen can follow what was done.
pub async fn wait<T: Send + 'static>(
    ctx: &Arc<AppContext>,
    label: &str,
    job: impl FnOnce(&AppServices, &Operation) -> T + Send + 'static,
) -> Result<T, IpcError> {
    let (sender, receiver) = mpsc::channel();
    start(ctx, label, Lane::Library, false, move |services, operation| {
        let _ = sender.send(job(services, operation));
    })?;
    blocking(move || receiver.recv().map_err(IpcError::internal)).await
}

/// Runs `action` for every document as one library operation "{step}: {n documents}" with progress per document,
/// and says how many succeeded (Kotlin `BulkActions.run`). Returns whether it started; a busy library does nothing.
pub fn run_each(
    ctx: &Arc<AppContext>,
    documents: &[&Document],
    step: &str,
    action: impl Fn(&AppServices, &Document) -> OperationOutcome + Send + 'static,
) -> Result<bool, IpcError> {
    let documents: Vec<Document> = documents.iter().map(|document| (*document).clone()).collect();
    let total = documents_text(documents.len());
    let step_owned = step.to_owned();
    let started = start(ctx, &format!("{step}: {total}"), Lane::Library, false, move |services, operation| {
        let succeeded = operation.for_each_with_progress(
            &step_owned,
            &documents,
            |document| document.path.value().to_owned(),
            |document| matches!(action(services, document), OperationOutcome::Done { .. }),
        );
        let text = format!("{step_owned}: {} of {total} done", count(succeeded));
        operation.session().report(if succeeded < documents.len() {
            UiMessage::error(text)
        } else {
            UiMessage::info(text)
        });
    });
    match started {
        Ok(()) => Ok(true),
        Err(error) if error.code == "busy" => Ok(false),
        Err(error) => Err(error),
    }
}
