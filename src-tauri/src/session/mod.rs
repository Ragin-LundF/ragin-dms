//! The open library as every screen sees it, and the one place that runs operations changing the library
//! (Kotlin's `LibrarySession`). Each operation runs in a lane: while one runs, others in its lane are refused, so
//! two of them never write the same files at once, but an AI run and a change of the library run side by side.

mod log;
mod model;
pub mod reporting;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub use log::SessionLog;
pub use model::{
    Lane, LibraryStatus, LogEntry, OperationProgress, RunningOperation, SessionFolder, SessionState, UiMessage,
};

/// Progress of a lane is pushed at most this often; a scan reports every file.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(80);

/// What the session tells the window. Implemented by the Tauri event adapter, recorded in tests.
pub trait SessionEvents: Send + Sync {
    fn state(&self, state: &SessionState);
    fn progress(&self, progress: &HashMap<Lane, OperationProgress>);
    fn log(&self, entry: &LogEntry);
    fn message(&self, message: &UiMessage);
}

pub struct Session {
    state: Mutex<SessionState>,
    progress: Mutex<Progress>,
    running: Mutex<HashMap<Lane, Arc<AtomicBool>>>,
    log: SessionLog,
    events: Arc<dyn SessionEvents>,
}

#[derive(Default)]
struct Progress {
    by_lane: HashMap<Lane, OperationProgress>,
    last_push: Option<Instant>,
}

/// The handle a running operation gets: progress, cancellation, log and messages.
pub struct Operation {
    session: Arc<Session>,
    lane: Lane,
    cancelled: Arc<AtomicBool>,
}

impl Session {
    pub fn new(events: Arc<dyn SessionEvents>) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(SessionState::default()),
            progress: Mutex::new(Progress::default()),
            running: Mutex::new(HashMap::new()),
            log: SessionLog::default(),
            events,
        })
    }

    pub fn state(&self) -> SessionState {
        lock(&self.state).clone()
    }

    pub fn progress(&self) -> HashMap<Lane, OperationProgress> {
        lock(&self.progress).by_lane.clone()
    }

    pub fn log(&self) -> &SessionLog {
        &self.log
    }

    pub fn info(&self, text: impl Into<String>) {
        self.events.log(&self.log.info(text));
    }

    pub fn error(&self, text: impl Into<String>) {
        self.events.log(&self.log.error(text));
    }

    #[allow(clippy::needless_pass_by_value)] // callers build the message in place
    pub fn report(&self, message: UiMessage) {
        self.events.message(&message);
    }

    /// Changes the shared state and tells the window.
    pub fn update(&self, change: impl FnOnce(&mut SessionState)) {
        let snapshot = {
            let mut state = lock(&self.state);
            change(&mut state);
            state.clone()
        };
        self.events.state(&snapshot);
    }

    /// Tells the screens that something they show changed, such as a new suggestion, so they reload it now
    /// instead of when the running operation ends.
    pub fn changed(&self) {
        self.update(|state| state.revision = state.revision.wrapping_add(1));
    }

    /// Runs `job` on its own thread as the one operation of its `lane`. Returns `false` and does nothing while
    /// another operation runs in that lane. Only a `cancellable` operation can be stopped by the user: one that,
    /// like a scan, leaves the library consistent wherever it stops; the job checks [`Operation::is_cancelled`].
    /// A job's `Err` is reported as "`label` failed: `detail`".
    pub fn perform<F>(self: &Arc<Self>, label: &str, lane: Lane, cancellable: bool, job: F) -> bool
    where
        F: FnOnce(&Operation) -> Result<(), String> + Send + 'static,
    {
        let operation = RunningOperation { label: label.to_owned(), lane, cancellable };
        if !self.claim(&operation) {
            return false;
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        lock(&self.running).insert(lane, Arc::clone(&cancelled));
        self.info(format!("{label}…"));
        let session = Arc::clone(self);
        std::thread::spawn(move || {
            let started = Instant::now();
            let handle = Operation { session: Arc::clone(&session), lane, cancelled };
            let outcome = job(&handle);
            session.finish(&operation, handle.is_cancelled(), outcome, started.elapsed());
        });
        true
    }

    /// Stops the operation running in `lane` if it can be stopped safely; see [`Self::perform`].
    pub fn cancel(&self, lane: Lane) {
        let label = {
            let state = lock(&self.state);
            let Some(operation) = state.operations.iter().find(|op| op.lane == lane && op.cancellable) else {
                return;
            };
            operation.label.clone()
        };
        self.info(format!("Cancelling: {label}"));
        if let Some(flag) = lock(&self.running).get(&lane) {
            flag.store(true, Ordering::SeqCst);
        }
    }

    fn claim(&self, operation: &RunningOperation) -> bool {
        let snapshot = {
            let mut state = lock(&self.state);
            if state.busy_in(operation.lane).is_some() {
                return false;
            }
            state.operations.push(operation.clone());
            state.clone()
        };
        self.events.state(&snapshot);
        true
    }

    fn finish(&self, operation: &RunningOperation, cancelled: bool, outcome: Result<(), String>, took: Duration) {
        let label = &operation.label;
        if cancelled {
            self.info(format!("{label}: cancelled"));
            self.report(UiMessage::info(format!("{label}: cancelled")));
        } else {
            match outcome {
                Ok(()) => self.info(format!("{label}: finished in {:.1} s", took.as_secs_f64())),
                Err(detail) => {
                    self.error(format!("{label} failed: {detail}"));
                    self.report(UiMessage::error(format!("{label} failed: {detail}")));
                }
            }
        }
        lock(&self.running).remove(&operation.lane);
        let progress = {
            let mut progress = lock(&self.progress);
            progress.by_lane.remove(&operation.lane);
            progress.by_lane.clone()
        };
        self.events.progress(&progress);
        self.update(|state| {
            state.operations.retain(|running| running != operation);
            state.revision = state.revision.wrapping_add(1);
        });
    }

    fn set_progress(&self, lane: Lane, value: OperationProgress) {
        let pushed = {
            let mut progress = lock(&self.progress);
            progress.by_lane.insert(lane, value);
            let due = progress.last_push.is_none_or(|last| last.elapsed() >= PROGRESS_INTERVAL);
            if due {
                progress.last_push = Some(Instant::now());
            }
            due.then(|| progress.by_lane.clone())
        };
        if let Some(progress) = pushed {
            self.events.progress(&progress);
        }
    }
}

impl Operation {
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Runs a blocking `call` that cannot check for cancellation itself, such as one request to the AI server, and
    /// returns `None` as soon as the operation is cancelled. The call then ends on its own thread and its result is
    /// dropped, so Cancel frees the lane at once instead of after the server's timeout.
    pub fn run_until_cancelled<T: Send + 'static>(&self, call: impl FnOnce() -> T + Send + 'static) -> Option<T> {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            // A receiver that stopped waiting is fine: the result is no longer wanted.
            let _ = sender.send(call());
        });
        loop {
            match receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(result) => return Some(result),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) if !self.is_cancelled() => {}
                Err(_) => return None,
            }
        }
    }

    /// Shows how far the operation is; cleared when it ends.
    pub fn progress(&self, progress: OperationProgress) {
        self.session.set_progress(self.lane, progress);
    }

    pub fn session(&self) -> &Arc<Session> {
        &self.session
    }

    /// Runs `action` for every item, one after the other, with progress and a log line each. Returns how many
    /// succeeded; stops early when the operation is cancelled.
    pub fn for_each_with_progress<T>(
        &self,
        step: &str,
        items: &[T],
        name: impl Fn(&T) -> String,
        mut action: impl FnMut(&T) -> bool,
    ) -> usize {
        let mut succeeded = 0;
        for (index, item) in items.iter().enumerate() {
            if self.is_cancelled() {
                break;
            }
            self.progress(OperationProgress::new(step, index, Some(items.len())).with_detail(name(item)));
            if action(item) {
                succeeded += 1;
                self.session.info(format!("{step}: {}", name(item)));
            } else {
                self.session.error(format!("{step} failed: {}", name(item)));
            }
        }
        succeeded
    }
}

/// A poisoned lock only means another thread panicked while holding it; the data is still the last state.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests;
