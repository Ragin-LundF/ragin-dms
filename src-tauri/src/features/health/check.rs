//! The archive check as a session operation: started by "Check now", the menu and the palette, or on its own after
//! the library opened when the schedule says it is due.

use std::sync::Arc;
use std::time::Duration;

use dms_application::health::VerifyProgress;
use dms_application::port::health::HealthReport;
use dms_application::services::AppServices;

use super::view::check_due;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::messages;
use crate::session::reporting::{Reporting, busy};
use crate::session::{Lane, LibraryStatus, Operation, OperationProgress};

const LABEL: &str = "Checking the archive";
/// The pause between two files of a check that started on its own, so the computer stays responsive.
const BACKGROUND_PAUSE: Duration = Duration::from_millis(30);
/// How long a due check waits for the library lane after opening before it gives up until the next opening.
const START_ATTEMPTS: u32 = 120;

impl AppContext {
    /// Starts the archive check in the library lane; it can be cancelled and continues where it stopped. A
    /// `background` check (one that started on its own) yields to every other library operation: it stops, lets
    /// the operation run, and starts again once the lane is free.
    ///
    /// # Errors
    /// `busy` while another library operation runs.
    pub fn check_archive(self: &Arc<Self>, background: bool) -> Result<(), IpcError> {
        let ctx = Arc::clone(self);
        let job = move |operation: &Operation| {
            let result = ctx.run_check(operation, background);
            if operation.is_preempted() {
                start_when_lane_is_free(&ctx);
            }
            result
        };
        let started = if background {
            self.session.perform_yielding(LABEL, Lane::Library, job)
        } else {
            self.session.perform(LABEL, Lane::Library, true, job)
        };
        if started { Ok(()) } else { Err(busy(&self.session, Lane::Library, LABEL)) }
    }

    fn run_check(&self, operation: &Operation, background: bool) -> Result<(), String> {
        let services = self.require_services().map_err(|error| error.message)?;
        let progress = Reporting::new(operation, |step: &VerifyProgress| {
            operation
                .progress(OperationProgress::new(LABEL, step.done, Some(step.total)).with_detail(step.current.value()));
            if background {
                std::thread::sleep(BACKGROUND_PAUSE);
            }
        });
        let outcome = services.sync.verify_library.run(&progress);
        if operation.is_preempted() {
            return Ok(());
        }
        if let Some(message) = messages::verify(&outcome) {
            operation.session().report(message);
        }
        operation.session().changed();
        Ok(())
    }
}

/// After the library opened: starts the check in the background when it is due, as soon as the library lane is
/// free. Call at the end of the opening operation.
pub fn start_when_due(ctx: &Arc<AppContext>) {
    let Some(services) = ctx.services() else { return };
    if !check_due(
        services.library.settings.load_preferences().unwrap_or_default().health_check,
        report_of_open_library(services).as_ref(),
        chrono::Utc::now(),
    ) {
        return;
    }
    start_when_lane_is_free(ctx);
}

/// Starts a background check as soon as the library lane is free, trying once a second for two minutes.
fn start_when_lane_is_free(ctx: &Arc<AppContext>) {
    let ctx = Arc::clone(ctx);
    std::thread::spawn(move || {
        for _ in 0..START_ATTEMPTS {
            // The opening operation (or the one the check made room for) still holds the lane; wait until it and
            // whatever follows are done.
            std::thread::sleep(Duration::from_secs(1));
            if !matches!(ctx.session.state().status, LibraryStatus::Open { .. }) || ctx.check_archive(true).is_ok() {
                return;
            }
        }
    });
}

/// The stored report when it belongs to the open library.
pub fn report_of_open_library(services: &AppServices) -> Option<HealthReport> {
    let catalog = services.library.store.catalog()?;
    services.sync.health.load().ok().flatten().filter(|report| &report.library_id == catalog.library_id())
}
