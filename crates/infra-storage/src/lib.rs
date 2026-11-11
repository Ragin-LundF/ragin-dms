//! The storage adapter of ragin-dms, ported from the Kotlin module `dms-storage`: the library folders on disk
//! (files, catalog, scanning, text store, vector index, restructure records, exports, folder watcher) and the
//! state of this device in the app data directory. Every file keeps the format the Kotlin version wrote.

mod catalog;
mod enum_names;
mod export;
mod fs;
mod index;
mod instant;
mod plan;
mod platform;
mod scan;
mod state;
mod storage_adapters;
mod text;
mod watch;
mod yaml;

pub use catalog::FileCatalogRepository;
pub use export::FileExportWriter;
pub use fs::FileSystemDocumentStorage;
pub use index::FileVectorIndex;
pub use plan::FilePlanStore;
pub use platform::{AppDataDirectory, PlatformTrash, SystemTrash};
pub use scan::FileSystemLibraryScanner;
pub use state::{FileActivityLog, FileOperationJournal, FileSettingsRepository, FileTaskStore};
pub use storage_adapters::StorageAdapters;
pub use text::FileTextStore;
pub use watch::FileSystemFolderWatcher;
