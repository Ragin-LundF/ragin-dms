//! Local state in the app data directory: settings, journal, tasks and activity.

pub mod dto;
mod file_activity_log;
mod file_operation_journal;
mod file_settings_repository;
mod file_task_store;
mod operation_mapper;
mod suggestion_mapper;

pub use file_activity_log::FileActivityLog;
pub use file_operation_journal::FileOperationJournal;
pub use file_settings_repository::FileSettingsRepository;
pub use file_task_store::FileTaskStore;
