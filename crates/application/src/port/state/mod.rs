//! Local state of this device: settings, preferences, the dashboard's task state and the activity log.

mod activity_log;
pub mod model;
mod settings_repository;
mod task_store;

pub use activity_log::ActivityLog;
pub use model::{ActivityEntry, ActivityKind, AiProvider, AiSettings, LayoutPreferences, Preferences};
pub use settings_repository::SettingsRepository;
pub use task_store::TaskStore;
