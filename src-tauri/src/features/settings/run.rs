//! Starts a session operation for a settings or inbox command (Kotlin `session.perform`).

use std::sync::Arc;

use crate::context::AppContext;
use crate::dto::IpcError;
use crate::session::reporting::busy;
use crate::session::{Lane, Operation};

/// Runs `job` as the operation `label` in `lane`; `Ok(false)` when the lane runs another operation.
pub(crate) fn try_perform(
    ctx: &Arc<AppContext>,
    label: &str,
    lane: Lane,
    job: impl FnOnce(&Arc<AppContext>, &Operation) -> Result<(), String> + Send + 'static,
) -> bool {
    let app = Arc::clone(ctx);
    ctx.session.perform(label, lane, false, move |operation| job(&app, operation))
}

/// Like [`try_perform`], with `busy` as the error while the lane runs another operation.
pub(crate) fn perform(
    ctx: &Arc<AppContext>,
    label: &str,
    lane: Lane,
    job: impl FnOnce(&Arc<AppContext>, &Operation) -> Result<(), String> + Send + 'static,
) -> Result<(), IpcError> {
    if try_perform(ctx, label, lane, job) { Ok(()) } else { Err(busy(&ctx.session, lane, label)) }
}

/// The services inside an operation; their absence is the operation's error.
pub(crate) fn services(ctx: &AppContext) -> Result<&Arc<dms_application::services::AppServices>, String> {
    ctx.require_services().map_err(|error| error.message)
}
