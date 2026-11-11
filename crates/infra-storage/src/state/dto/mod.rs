//! The files of the app data directory, as the Kotlin DTOs bound them. These files keep no fields of newer
//! versions: the Kotlin version dropped them on the next write as well.

mod activity_entry_dto;
mod ai_settings_dto;
mod layout_dto;
mod library_folder_dto;
mod settings_dto;
mod suggested_tag_dto;
mod suggestion_dto;

pub use activity_entry_dto::ActivityEntryDto;
pub use ai_settings_dto::AiSettingsDto;
pub use layout_dto::LayoutDto;
pub use library_folder_dto::LibraryFolderDto;
pub use settings_dto::SettingsDto;
pub use suggested_tag_dto::SuggestedTagDto;
pub use suggestion_dto::SuggestionDto;
