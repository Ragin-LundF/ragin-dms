//! The shell: shared state, progress, log, cancel, and the platform's file manager.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use tauri::AppHandle;

use super::SessionState as AppSession;
use crate::events::LaneProgress;
use crate::session::{Lane, LogEntry, SessionState};

#[tauri::command]
#[specta::specta]
pub fn session_state(session: AppSession<'_>) -> SessionState {
    session.state()
}

#[tauri::command]
#[specta::specta]
pub fn session_progress(session: AppSession<'_>) -> Vec<LaneProgress> {
    session.progress().into_iter().map(|(lane, progress)| LaneProgress { lane, progress }).collect()
}

#[tauri::command]
#[specta::specta]
pub fn log_entries(session: AppSession<'_>) -> Vec<LogEntry> {
    session.log().entries()
}

#[tauri::command]
#[specta::specta]
pub fn clear_log(session: AppSession<'_>) {
    session.log().clear();
}

/// Stops the operation running in `lane` if it may be stopped.
#[tauri::command]
#[specta::specta]
pub fn cancel_operation(session: AppSession<'_>, lane: Lane) {
    session.cancel(lane);
}

/// Which menu entries apply: the "Documents" menu while the library screen is shown, and the log's check mark.
#[tauri::command]
#[specta::specta]
pub fn set_menu_context(app: AppHandle, library_available: bool, log_visible: bool) {
    crate::menu::set_context(&app, library_available, log_visible);
}

/// "Finder", "Explorer" or "file manager", for labels such as "Show in Finder".
#[tauri::command]
#[specta::specta]
pub fn file_manager_name() -> String {
    if cfg!(target_os = "macos") {
        "Finder"
    } else if cfg!(target_os = "windows") {
        "Explorer"
    } else {
        "file manager"
    }
    .to_owned()
}

/// The system locale (BCP 47, e.g. `de-CH`), so counts group their digits as Kotlin's `%,d` did with the JVM's
/// default locale; `en-US` when the system names none.
#[tauri::command]
#[specta::specta]
pub fn number_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "en-US".to_owned())
}
