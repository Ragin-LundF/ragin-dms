//! The system trash and the app data directory of this device.

mod app_data_directory;
mod platform_trash;
mod system_trash;

pub use app_data_directory::AppDataDirectory;
pub use platform_trash::PlatformTrash;
pub use system_trash::SystemTrash;
