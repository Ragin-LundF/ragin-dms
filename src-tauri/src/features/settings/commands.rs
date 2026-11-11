//! The Settings screen's commands (Kotlin `SettingsViewModel`, `LibrarySettingsController`).
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;

use dms_application::port::state::Preferences;

use super::ai::{ai_settings_of, check_draft, present_connection};
use super::dto::{
    AiConnectionDto, AiDraftCheckDto, AiDraftDto, AiSettingsDto, InboxSettingsDto, LibraryCommandDto,
    LibrarySettingsChangeDto, LibrarySettingsDto, SettingsStateDto,
};
use super::folders::{configure_message, library_command, present_settings};
use super::library_settings::{
    SettingsPlan, inbox_path_problem, inbox_saved_message, inbox_settings_of, plan_change, present_library_settings,
};
use super::run::{perform, services};
use crate::commands::AppState;
use crate::dto::IpcError;
use crate::messages;
use crate::session::Lane;
use crate::session::reporting::blocking;

/// The library folders of this device and its preferences.
#[tauri::command]
#[specta::specta]
pub async fn settings_state(ctx: AppState<'_>) -> Result<SettingsStateDto, IpcError> {
    let app = Arc::clone(&ctx);
    blocking(move || {
        let settings = &app.require_services()?.library.settings;
        let unreadable = |error: dms_application::port::storage::StorageError| {
            IpcError::internal(format!("The settings could not be read ({})", messages::failure(error.reason)))
        };
        let folders = settings.load_library_folders().map_err(unreadable)?;
        let preferences = settings.load_preferences().map_err(unreadable)?;
        Ok(present_settings(folders.as_ref(), &app.session.state().status, &preferences))
    })
    .await
}

/// The settings stored in the catalog; `None` while no library is open.
#[tauri::command]
#[specta::specta]
pub fn library_settings(ctx: AppState<'_>) -> Option<LibrarySettingsDto> {
    ctx.catalog().map(|catalog| present_library_settings(&catalog))
}

/// Chooses the primary folder, adds, removes or promotes a sync folder, as the operation "Updating the library
/// folders"; on success the library is reopened.
#[tauri::command]
#[specta::specta]
pub fn configure_library(ctx: AppState<'_>, command: LibraryCommandDto) -> Result<(), IpcError> {
    let command = library_command(command)?;
    perform(&ctx, "Updating the library folders", Lane::Library, move |app, operation| {
        match configure_message(&services(app)?.library.configure_library.run(&command)) {
            None => app.reopen(operation),
            Some(message) => {
                operation.session().report(message);
                Ok(())
            }
        }
    })
}

/// "Add new files while scanning"; saved as the operation "Saving the preferences".
#[tauri::command]
#[specta::specta]
pub fn save_add_new_files_while_scanning(ctx: AppState<'_>, enabled: bool) -> Result<(), IpcError> {
    ctx.update_preferences(move |preferences| Preferences { add_new_files_while_scanning: enabled, ..preferences })
}

/// Saves the AI settings of this device as the operation "Saving the preferences".
#[tauri::command]
#[specta::specta]
pub fn save_ai_settings(ctx: AppState<'_>, settings: AiSettingsDto) -> Result<(), IpcError> {
    let ai = ai_settings_of(settings)?;
    ctx.update_preferences(move |preferences| Preferences { ai, ..preferences })
}

/// What the typed AI settings are worth compared with the `saved` ones.
#[tauri::command]
#[specta::specta]
pub fn check_ai_draft(draft: AiDraftDto, saved: AiSettingsDto) -> AiDraftCheckDto {
    check_draft(&draft, &saved)
}

/// Asks the server for its models with the typed `settings`, without saving them.
#[tauri::command]
#[specta::specta]
pub async fn test_ai_connection(ctx: AppState<'_>, settings: AiSettingsDto) -> Result<AiConnectionDto, IpcError> {
    let settings = ai_settings_of(settings)?;
    let app = Arc::clone(&ctx);
    blocking(move || Ok(present_connection(&app.require_services()?.ai.list_ai_models.run(&settings)))).await
}

/// Changes the catalog settings (fields, currency, embedded metadata, rules) as the operation "Saving the library
/// settings"; a change that cannot be made is reported as a message.
#[tauri::command]
#[specta::specta]
pub fn change_library_settings(ctx: AppState<'_>, change: LibrarySettingsChangeDto) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    match plan_change(change, catalog.settings())? {
        SettingsPlan::Nothing => Ok(()),
        SettingsPlan::Refused(message) => {
            ctx.session.report(message);
            Ok(())
        }
        SettingsPlan::Change(change) => {
            perform(&ctx, "Saving the library settings", Lane::Library, move |app, operation| {
                let outcome =
                    services(app)?.documents.change_settings.run(&change.subject, |settings| (change.apply)(settings));
                operation.session().report(messages::of(&outcome, &change.done));
                Ok(())
            })
        }
    }
}

/// Why `path` cannot be the inbox, or `None` if it can.
#[tauri::command]
#[specta::specta]
pub fn check_inbox_path(path: String) -> Option<String> {
    inbox_path_problem(&path).map(str::to_owned)
}

/// Saves the inbox settings as "Saving the inbox settings"; switching it on creates the folder in every library
/// folder, then the library is scanned.
#[tauri::command]
#[specta::specta]
pub fn save_inbox(ctx: AppState<'_>, inbox: InboxSettingsDto) -> Result<(), IpcError> {
    let Some(inbox) = inbox_settings_of(&inbox) else {
        return Err(IpcError::invalid(inbox_path_problem(&inbox.path).unwrap_or("Invalid inbox path")));
    };
    perform(&ctx, "Saving the inbox settings", Lane::Library, move |app, operation| {
        let outcome = services(app)?.organize.configure_inbox.run(&inbox);
        operation.session().report(messages::of(&outcome, &inbox_saved_message(&inbox)));
        app.rescan(operation)
    })
}
