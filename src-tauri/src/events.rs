//! Events Rust pushes to the window. The session's are sent through [`TauriSessionEvents`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tauri_specta::Event;

use crate::session::{Lane, LogEntry, OperationProgress, SessionEvents, SessionState, UiMessage};

/// The shared state changed: an operation started or ended, the library opened, or something was saved.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct SessionStateEvent {
    pub state: SessionState,
}

/// How far the running operations are, one entry per busy lane.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub lanes: Vec<LaneProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LaneProgress {
    pub lane: Lane,
    pub progress: OperationProgress,
}

/// A new line of the session log.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub entry: LogEntry,
}

/// A message for the snackbar, such as the result of an action.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct MessageEvent {
    pub message: UiMessage,
}

/// The user chose an entry of the native menu bar; `id` is one of [`crate::menu::MenuId`]'s names.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct MenuEvent {
    pub id: String,
}

pub struct TauriSessionEvents {
    handle: AppHandle,
}

impl TauriSessionEvents {
    pub fn new(handle: AppHandle) -> Self {
        Self { handle }
    }
}

// A closed window cannot receive events; there is nothing to recover, so emit errors are ignored.
impl SessionEvents for TauriSessionEvents {
    fn state(&self, state: &SessionState) {
        crate::menu::follow_session(&self.handle, state);
        let _ = SessionStateEvent { state: state.clone() }.emit(&self.handle);
    }

    fn progress(&self, progress: &HashMap<Lane, OperationProgress>) {
        let lanes = progress.iter().map(|(lane, progress)| LaneProgress { lane: *lane, progress: progress.clone() });
        let _ = ProgressEvent { lanes: lanes.collect() }.emit(&self.handle);
    }

    fn log(&self, entry: &LogEntry) {
        // The terminal of `npm run tauri dev` (or the console) is where a developer reads the session log.
        eprintln!("{}  {}", entry.at, entry.text);
        let _ = LogEvent { entry: entry.clone() }.emit(&self.handle);
    }

    fn message(&self, message: &UiMessage) {
        let _ = MessageEvent { message: message.clone() }.emit(&self.handle);
    }
}
