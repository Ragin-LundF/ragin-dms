//! The library folders of this device as the Settings screen shows them, and the changes to them (Kotlin
//! `SettingsViewModel`'s folder part).

use dms_application::library::model::LibraryCommand;
use dms_application::library::model::outcome::ConfigureLibraryOutcome;
use dms_application::port::state::Preferences;
use dms_domain::library::{FolderLocation, FolderRole, LibraryFolderId, LibraryFolders};

use super::ai::ai_settings_dto;
use super::dto::{FolderRowDto, LibraryCommandDto, SettingsStateDto};
use crate::dto::IpcError;
use crate::messages;
use crate::session::{LibraryStatus, UiMessage};

/// The device part of Settings: one row per folder in settings order, with a note while the library is open and
/// leaves the folder alone.
pub fn present_settings(
    folders: Option<&LibraryFolders>,
    status: &LibraryStatus,
    preferences: &Preferences,
) -> SettingsStateDto {
    let rows: Vec<FolderRowDto> = folders.map_or_else(Vec::new, |folders| {
        folders
            .folders()
            .iter()
            .map(|folder| {
                let id = folder.id.value();
                FolderRowDto {
                    id: id.to_owned(),
                    display_name: folder.display_name.clone(),
                    location: folder.location.value().to_owned(),
                    primary: folder.role == FolderRole::Primary,
                    note: note_of(id, status).map(str::to_owned),
                }
            })
            .collect()
    });
    SettingsStateDto {
        configured: !rows.is_empty(),
        folders: rows,
        add_new_files_while_scanning: preferences.add_new_files_while_scanning,
        ai: ai_settings_dto(&preferences.ai),
    }
}

fn note_of(folder_id: &str, status: &LibraryStatus) -> Option<&'static str> {
    let LibraryStatus::Open { foreign_folders, unreachable_folders } = status else {
        return None;
    };
    if foreign_folders.iter().any(|id| id == folder_id) {
        return Some("Holds another library; left alone");
    }
    unreachable_folders.iter().any(|id| id == folder_id).then_some("Not reachable when the library was opened")
}

/// The use case's command for what the user chose.
///
/// # Errors
/// `invalid_input` for a blank location or folder id.
pub fn library_command(command: LibraryCommandDto) -> Result<LibraryCommand, IpcError> {
    let invalid = |error: dms_domain::DomainError| IpcError::invalid(error.to_string());
    Ok(match command {
        LibraryCommandDto::CreatePrimary { location } => LibraryCommand::CreatePrimary {
            display_name: display_name_of(&location),
            location: FolderLocation::new(location).map_err(invalid)?,
        },
        LibraryCommandDto::AddSyncFolder { location } => LibraryCommand::AddSyncFolder {
            display_name: display_name_of(&location),
            location: FolderLocation::new(location).map_err(invalid)?,
        },
        LibraryCommandDto::RemoveSyncFolder { folder_id } => {
            LibraryCommand::RemoveSyncFolder(LibraryFolderId::new(folder_id).map_err(invalid)?)
        }
        LibraryCommandDto::ChoosePrimary { folder_id } => {
            LibraryCommand::ChoosePrimary(LibraryFolderId::new(folder_id).map_err(invalid)?)
        }
    })
}

/// The name of a new folder: the last segment of its location, or the whole location if that is blank.
pub fn display_name_of(location: &str) -> String {
    let trimmed = location.trim_end_matches(['/', '\\']);
    let name = trimmed.rsplit('/').next().unwrap_or(trimmed);
    let name = name.rsplit('\\').next().unwrap_or(name);
    if dms_domain::text::is_blank(name) { location.to_owned() } else { name.to_owned() }
}

/// The message for a folder change; `None` when it worked and the library is reopened (Kotlin
/// `OutcomeMessages.configure`).
// shortcut: lives here until Bob moves it to messages.rs.
pub fn configure_message(outcome: &ConfigureLibraryOutcome) -> Option<UiMessage> {
    let text = match outcome {
        ConfigureLibraryOutcome::Configured(_) => return None,
        ConfigureLibraryOutcome::AlreadyConfigured => "A primary folder is already configured".to_owned(),
        ConfigureLibraryOutcome::NotConfigured => "Choose a primary folder first".to_owned(),
        ConfigureLibraryOutcome::LocationAlreadyUsed => "This folder is already part of the library".to_owned(),
        ConfigureLibraryOutcome::UnknownFolder => "This folder is no longer configured".to_owned(),
        ConfigureLibraryOutcome::PrimaryCannotBeRemoved => {
            "The primary folder cannot be removed; make another folder the primary first".to_owned()
        }
        // New in Rust: Kotlin threw for both.
        ConfigureLibraryOutcome::InvalidInput(error) => error.to_string(),
        ConfigureLibraryOutcome::StorageFailed(error) => {
            format!("The settings could not be saved ({}); nothing was changed", messages::failure(error.reason))
        }
    };
    Some(UiMessage::error(text))
}

#[cfg(test)]
mod tests {
    use dms_application::port::state::Preferences;
    use dms_domain::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId, LibraryFolders};

    use super::*;

    fn folder(id: &str, name: &str, role: FolderRole) -> LibraryFolder {
        LibraryFolder::new(
            LibraryFolderId::new(id).expect("id"),
            name,
            FolderLocation::new(format!("/Synthetic/{name}")).expect("location"),
            role,
        )
        .expect("folder")
    }

    fn folders() -> LibraryFolders {
        LibraryFolders::new(vec![
            folder("a", "Documents", FolderRole::Primary),
            folder("b", "Cloud", FolderRole::Sync),
            folder("c", "Stick", FolderRole::Sync),
        ])
        .expect("folders")
    }

    #[test]
    fn rows_keep_settings_order_and_note_foreign_and_unreachable_folders_while_open() {
        let status =
            LibraryStatus::Open { foreign_folders: vec!["b".to_owned()], unreachable_folders: vec!["c".to_owned()] };
        let state = present_settings(Some(&folders()), &status, &Preferences::default());
        assert!(state.configured);
        let notes: Vec<_> =
            state.folders.iter().map(|row| (row.id.as_str(), row.primary, row.note.as_deref())).collect();
        assert_eq!(
            notes,
            vec![
                ("a", true, None),
                ("b", false, Some("Holds another library; left alone")),
                ("c", false, Some("Not reachable when the library was opened")),
            ]
        );
    }

    #[test]
    fn notes_are_shown_only_while_the_library_is_open() {
        let status = LibraryStatus::PrimaryFailed { reason: "folder not reachable".to_owned(), detail: None };
        let state = present_settings(Some(&folders()), &status, &Preferences::default());
        assert!(state.folders.iter().all(|row| row.note.is_none()));
    }

    #[test]
    fn without_folders_the_library_is_not_configured() {
        let state = present_settings(None, &LibraryStatus::NotConfigured, &Preferences::default());
        assert!(!state.configured);
        assert!(state.add_new_files_while_scanning);
    }

    #[test]
    fn display_name_is_the_last_segment_of_either_separator() {
        assert_eq!(display_name_of("/Users/someone/Documents/"), "Documents");
        assert_eq!(display_name_of("C:\\Data\\Archive\\"), "Archive");
        assert_eq!(display_name_of("/"), "/");
    }

    #[test]
    fn a_blank_location_is_refused() {
        let refused = library_command(LibraryCommandDto::AddSyncFolder { location: " ".to_owned() });
        assert!(refused.is_err_and(|error| error.code == "invalid_input"));
    }

    #[test]
    fn a_working_folder_change_reports_nothing_and_refusals_are_phrased() {
        assert!(configure_message(&ConfigureLibraryOutcome::Configured(folders())).is_none());
        let message = configure_message(&ConfigureLibraryOutcome::PrimaryCannotBeRemoved).expect("message");
        assert!(message.is_error);
        assert_eq!(message.text, "The primary folder cannot be removed; make another folder the primary first");
    }
}
