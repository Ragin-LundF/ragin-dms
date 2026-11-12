use std::path::PathBuf;

use dms_application::port::state::{AiSettings, LayoutPreferences, Preferences, SettingsRepository};
use dms_application::port::storage::StorageResult;
use dms_domain::library::{FolderLocation, LibraryFolder, LibraryFolderId, LibraryFolders};

use dms_domain::health::HealthSchedule;

use crate::enum_names::{self, AI_PROVIDERS, FOLDER_ROLES, HEALTH_SCHEDULES};
use crate::state::dto::{AiSettingsDto, LayoutDto, LibraryFolderDto, SettingsDto};
use crate::yaml::{YamlError, YamlFile};

/// `settings.yaml` in the app data folder. An unreadable file is an error, never "not configured": otherwise a
/// new configuration would silently replace the existing one. Writes are read-modify-write.
pub struct FileSettingsRepository {
    file: YamlFile,
}

impl FileSettingsRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { file: YamlFile::new(path) }
    }

    fn read(&self) -> StorageResult<Option<SettingsDto>> {
        Ok(self.file.read()?.map(|node| SettingsDto::read(&node)).transpose()?)
    }

    fn update(&self, change: impl FnOnce(SettingsDto) -> SettingsDto) -> StorageResult<()> {
        let current = self.read()?.unwrap_or_default();
        self.file.write(&change(current).write())
    }
}

impl SettingsRepository for FileSettingsRepository {
    fn load_library_folders(&self) -> StorageResult<Option<LibraryFolders>> {
        let folders = self.read()?.map(|settings| settings.folders).unwrap_or_default();
        if folders.is_empty() {
            return Ok(None);
        }
        let folders = folders.iter().map(folder_to_domain).collect::<Result<Vec<_>, _>>()?;
        Ok(Some(LibraryFolders::new(folders).map_err(YamlError::from)?))
    }

    fn save_library_folders(&self, folders: &LibraryFolders) -> StorageResult<()> {
        let folders: Vec<LibraryFolderDto> = folders.folders().iter().map(folder_to_dto).collect();
        self.update(|current| SettingsDto { folders, ..current })
    }

    fn load_preferences(&self) -> StorageResult<Preferences> {
        let settings = self.read()?.unwrap_or_default();
        let ai = &settings.ai;
        Ok(Preferences {
            add_new_files_while_scanning: settings.add_new_files_while_scanning,
            ai: AiSettings {
                enabled: ai.is_enabled(),
                base_url: ai.base_url.clone(),
                model: ai.model.clone(),
                prefer_vision_for_pdfs: ai.prefer_vision_for_pdfs,
                timeout_seconds: ai.timeout_seconds,
                accept_suggested_file_names: ai.accept_suggested_file_names,
                embedding_model: ai.embedding_model.clone(),
                // A hand-edited value out of range is moved into it instead of making the file unreadable.
                input_characters: coerce(ai.input_characters, &AiSettings::INPUT_CHARACTERS_RANGE),
                // An unknown provider falls back to the local server instead of making the file unreadable.
                provider: enum_names::value_of(AI_PROVIDERS, &ai.provider).unwrap_or_default(),
                anthropic_model: ai.anthropic_model.clone(),
            },
            layout: LayoutPreferences {
                sidebar_width: coerce(settings.layout.sidebar_width, &LayoutPreferences::SIDEBAR_WIDTHS),
                sidebar_folded: settings.layout.sidebar_folded,
                folder_tree_width: coerce(settings.layout.folder_tree_width, &LayoutPreferences::FOLDER_TREE_WIDTHS),
                folder_tree_hidden: settings.layout.folder_tree_hidden,
            },
            // An unknown schedule falls back to the default instead of making the file unreadable.
            health_check: settings
                .health_check
                .as_deref()
                .and_then(|name| enum_names::value_of(HEALTH_SCHEDULES, name))
                .unwrap_or_default(),
        })
    }

    fn save_preferences(&self, preferences: &Preferences) -> StorageResult<()> {
        let ai = &preferences.ai;
        let ai = AiSettingsDto {
            enabled: Some(ai.enabled),
            base_url: ai.base_url.clone(),
            model: ai.model.clone(),
            prefer_vision_for_pdfs: ai.prefer_vision_for_pdfs,
            timeout_seconds: ai.timeout_seconds,
            accept_suggested_file_names: ai.accept_suggested_file_names,
            embedding_model: ai.embedding_model.clone(),
            input_characters: ai.input_characters,
            provider: enum_names::name_of(AI_PROVIDERS, ai.provider).to_owned(),
            anthropic_model: ai.anthropic_model.clone(),
        };
        let layout = LayoutDto {
            sidebar_width: preferences.layout.sidebar_width,
            sidebar_folded: preferences.layout.sidebar_folded,
            folder_tree_width: preferences.layout.folder_tree_width,
            folder_tree_hidden: preferences.layout.folder_tree_hidden,
        };
        let add_new_files_while_scanning = preferences.add_new_files_while_scanning;
        let health_check = (preferences.health_check != HealthSchedule::default())
            .then(|| enum_names::name_of(HEALTH_SCHEDULES, preferences.health_check).to_owned());
        self.update(|current| SettingsDto { add_new_files_while_scanning, ai, layout, health_check, ..current })
    }
}

fn coerce(value: i32, range: &std::ops::RangeInclusive<i32>) -> i32 {
    value.clamp(*range.start(), *range.end())
}

fn folder_to_dto(folder: &LibraryFolder) -> LibraryFolderDto {
    LibraryFolderDto {
        id: folder.id.value().to_owned(),
        display_name: folder.display_name.clone(),
        location: folder.location.value().to_owned(),
        role: enum_names::name_of(FOLDER_ROLES, folder.role).to_owned(),
    }
}

fn folder_to_domain(folder: &LibraryFolderDto) -> Result<LibraryFolder, YamlError> {
    Ok(LibraryFolder::new(
        LibraryFolderId::new(folder.id.as_str())?,
        folder.display_name.clone(),
        FolderLocation::new(folder.location.as_str())?,
        enum_names::parse(FOLDER_ROLES, &folder.role)?,
    )?)
}
