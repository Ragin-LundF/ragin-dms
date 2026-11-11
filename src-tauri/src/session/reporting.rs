//! Bridges a use case's `Progress` to the running session operation, and helpers every feature command shares.

use dms_application::common::Progress;

use crate::session::Operation;

/// Calls `on_report` for every step and answers `is_cancelled` from the operation, so Cancel in the activity bar
/// stops the use case between two items.
pub(crate) struct Reporting<'a, F> {
    operation: &'a Operation,
    on_report: F,
}

impl<'a, F> Reporting<'a, F> {
    pub(crate) fn new(operation: &'a Operation, on_report: F) -> Self {
        Self { operation, on_report }
    }
}

impl<P, F: Fn(&P) + Sync> Progress<P> for Reporting<'_, F> {
    fn report(&self, progress: &P) {
        (self.on_report)(progress);
    }

    fn is_cancelled(&self) -> bool {
        self.operation.is_cancelled()
    }
}

/// The error of a command whose operation could not start because `lane` runs another one.
pub(crate) fn busy(session: &crate::session::Session, lane: crate::session::Lane, label: &str) -> crate::dto::IpcError {
    let state = session.state();
    let running = state.busy_in(lane).unwrap_or(label);
    crate::dto::IpcError::new("busy", crate::messages::wait_for(running))
}

/// Runs blocking work (a use case) off the command thread.
pub(crate) async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, crate::dto::IpcError> + Send + 'static,
) -> Result<T, crate::dto::IpcError> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(crate::dto::IpcError::internal)?
}
