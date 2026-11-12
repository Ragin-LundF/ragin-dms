//! The library-wide operations of the menu bar, the toolbar and the palette; each runs in the session's library
//! lane (Kotlin `LibrarySession.open/scan/syncToAllDestinations/...`).
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use serde::{Deserialize, Serialize};
use specta::Type;

use super::AppState;
use crate::dto::IpcError;

/// What the navigation badges show: documents in the inbox, open tasks, overdue bills and follow-ups due.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NavigationCounts {
    pub inbox: u32,
    pub tasks: u32,
    pub to_pay: u32,
    pub cases: u32,
}

/// Opens the library, merging the catalogs of all folders, and scans it.
#[tauri::command]
#[specta::specta]
pub fn open_library(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.inner().open()
}

/// Scans every folder; cancellable.
#[tauri::command]
#[specta::specta]
pub fn scan_library(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.inner().scan()
}

#[tauri::command]
#[specta::specta]
pub fn sync_to_all_destinations(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.inner().sync_to_all_destinations()
}

#[tauri::command]
#[specta::specta]
pub fn merge_all_folders(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.inner().merge_all_folders()
}

#[tauri::command]
#[specta::specta]
pub fn retry_failed_operations(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.inner().retry_failed_operations()
}

/// Gives documents without a date the one found in their file name or creation date.
#[tauri::command]
#[specta::specta]
pub fn detect_document_dates(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.inner().detect_document_dates()
}

#[tauri::command]
#[specta::specta]
pub fn navigation_counts(ctx: AppState<'_>) -> NavigationCounts {
    let count = |value: usize| u32::try_from(value).unwrap_or(u32::MAX);
    let inbox = ctx.catalog().map_or(0, |catalog| {
        let inbox = &catalog.settings().layout.inbox;
        if !inbox.enabled {
            return 0;
        }
        catalog.documents().values().filter(|document| inbox.contains(&document.path)).count()
    });
    let dashboard = ctx.dashboard();
    let tasks = dashboard.as_ref().map_or(&[][..], |dashboard| dashboard.tasks.as_slice());
    let badges = crate::features::organize::view::organize_badges(tasks);
    NavigationCounts {
        inbox: count(inbox),
        tasks: count(tasks.len()),
        to_pay: count(badges.to_pay),
        cases: count(badges.cases),
    }
}

/// Lets the AI suggest metadata for every inbox document without one (Kotlin `LibrarySession.analyzeInbox`).
#[tauri::command]
#[specta::specta]
pub fn analyze_inbox_documents(ctx: AppState<'_>) -> Result<(), IpcError> {
    ctx.inner().analyze_inbox()
}

/// Switches adding new files during a scan on or off; it applies from the next scan.
#[tauri::command]
#[specta::specta]
pub fn set_add_new_files_while_scanning(ctx: AppState<'_>, enabled: bool) -> Result<(), IpcError> {
    ctx.inner().update_preferences(move |preferences| dms_application::port::state::Preferences {
        add_new_files_while_scanning: enabled,
        ..preferences
    })
}
