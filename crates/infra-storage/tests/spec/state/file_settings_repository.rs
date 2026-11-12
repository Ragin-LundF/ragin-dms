use std::fs;

use dms_application::port::state::{AiProvider, AiSettings, LayoutPreferences, Preferences, SettingsRepository};
use dms_application::port::storage::StorageFailure;
use dms_domain::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId, LibraryFolders};
use dms_infra_storage::FileSettingsRepository;

use crate::support::{failure_of, fixture, read};

fn folder(id: &str, display_name: &str, location: &str, role: FolderRole) -> LibraryFolder {
    LibraryFolder::new(
        LibraryFolderId::new(id).expect("id"),
        display_name,
        FolderLocation::new(location).expect("location"),
        role,
    )
    .expect("folder")
}

#[test]
fn nothing_is_configured_before_the_first_save() {
    let temp = tempfile::tempdir().expect("temp dir");

    assert_eq!(
        FileSettingsRepository::new(temp.path().join("settings.yaml")).load_library_folders().expect("read"),
        None
    );
}

#[test]
fn the_library_folders_survive_a_restart_in_the_format_jackson_wrote() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("settings.yaml");
    let folders = LibraryFolders::new(vec![
        folder("primary", "Bibliothek: Haupt", "/data/Bibliothek 2024", FolderRole::Primary),
        folder("sync", "yes", "C:\\Users\\Example\\Library", FolderRole::Sync),
    ])
    .expect("folders");

    FileSettingsRepository::new(&path).save_library_folders(&folders).expect("saved");

    assert_eq!(FileSettingsRepository::new(&path).load_library_folders().expect("read"), Some(folders));
    assert_eq!(read(&path), fixture("jackson/settings-default.yaml"));
}

#[test]
fn preferences_default_to_adding_new_files_and_are_kept_apart_from_the_folders() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("settings.yaml");
    let repository = FileSettingsRepository::new(&path);
    let folders = LibraryFolders::new(vec![folder("primary", "Folder primary", "/data/library", FolderRole::Primary)])
        .expect("folders");
    assert_eq!(repository.load_preferences().expect("read"), Preferences::default());
    let preferences = Preferences {
        add_new_files_while_scanning: false,
        ai: AiSettings {
            base_url: "http://localhost:9999/v1".to_owned(),
            model: Some("sample-model".to_owned()),
            prefer_vision_for_pdfs: true,
            accept_suggested_file_names: true,
            input_characters: 30_000,
            provider: AiProvider::Anthropic,
            anthropic_model: "claude-sonnet-5-5".to_owned(),
            ..AiSettings::default()
        },
        layout: LayoutPreferences {
            sidebar_width: 280,
            sidebar_folded: true,
            folder_tree_width: 300,
            folder_tree_hidden: true,
        },
        ..Preferences::default()
    };

    repository.save_preferences(&preferences).expect("saved");
    repository.save_library_folders(&folders).expect("saved");

    let reloaded = FileSettingsRepository::new(&path);
    assert_eq!(reloaded.load_preferences().expect("read"), preferences);
    assert_eq!(reloaded.load_library_folders().expect("read"), Some(folders));
}

#[test]
fn saved_preferences_have_the_bytes_jackson_wrote() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("settings.yaml");
    let preferences = Preferences {
        add_new_files_while_scanning: false,
        ai: AiSettings {
            enabled: true,
            base_url: "http://localhost:9999/v1".to_owned(),
            model: Some("sample-model".to_owned()),
            prefer_vision_for_pdfs: true,
            accept_suggested_file_names: true,
            embedding_model: Some("embed: v2".to_owned()),
            input_characters: 30_000,
            provider: AiProvider::Anthropic,
            anthropic_model: "claude-sonnet-5-5".to_owned(),
            ..AiSettings::default()
        },
        layout: LayoutPreferences {
            sidebar_width: 280,
            sidebar_folded: true,
            folder_tree_width: 300,
            folder_tree_hidden: false,
        },
        ..Preferences::default()
    };

    FileSettingsRepository::new(&path).save_preferences(&preferences).expect("saved");

    assert_eq!(read(&path), fixture("jackson/settings-full.yaml"));
}

#[test]
fn an_unreadable_settings_file_is_an_error_not_an_empty_configuration() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("settings.yaml");
    fs::write(&path, "folders: [").expect("file");

    assert_eq!(
        failure_of(FileSettingsRepository::new(&path).load_library_folders()),
        Some(StorageFailure::UnsupportedFormat)
    );
}

#[test]
fn an_older_ai_block_or_an_unknown_provider_loads_as_local_ai_with_the_default_claude_model() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("settings.yaml");
    fs::write(&path, "ai:\n  baseUrl: http://localhost:9999/v1\n  provider: SOMETHING\n").expect("file");

    let ai = FileSettingsRepository::new(&path).load_preferences().expect("read").ai;

    assert_eq!(ai.provider, AiProvider::Local);
    assert_eq!(ai.anthropic_model, AiSettings::DEFAULT_ANTHROPIC_MODEL);
    assert_eq!(ai.base_url, "http://localhost:9999/v1");
}

#[test]
fn hand_edited_values_out_of_range_are_moved_into_it() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("settings.yaml");
    fs::write(&path, "ai:\n  inputCharacters: 5\nlayout:\n  sidebarWidth: 9999\n  folderTreeWidth: 1\n").expect("file");

    let preferences = FileSettingsRepository::new(&path).load_preferences().expect("read");

    assert_eq!(preferences.ai.input_characters, 1_000);
    assert_eq!(preferences.layout.sidebar_width, 360);
    assert_eq!(preferences.layout.folder_tree_width, 160);
}

#[test]
fn ai_is_off_on_a_new_device_on_for_an_older_file_whose_ai_settings_were_changed_and_kept_as_saved() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("settings.yaml");
    let enabled = || FileSettingsRepository::new(&path).load_preferences().expect("read").ai.enabled;
    assert!(!enabled());

    fs::write(&path, "ai:\n  model: sample-model\n").expect("file");
    assert!(enabled());
    fs::write(&path, "ai:\n  timeoutSeconds: 300\n").expect("file");
    assert!(!enabled());

    let repository = FileSettingsRepository::new(&path);
    let on = Preferences { ai: AiSettings { enabled: true, ..AiSettings::default() }, ..Preferences::default() };
    repository.save_preferences(&on).expect("saved");
    assert!(enabled());
    let model_only = Preferences {
        ai: AiSettings { model: Some("sample-model".to_owned()), ..AiSettings::default() },
        ..Preferences::default()
    };
    repository.save_preferences(&model_only).expect("saved");
    assert!(!enabled());
}
