use std::path::Path;
use std::sync::Arc;

use crate::catalog::FileCatalogRepository;
use crate::export::FileExportWriter;
use crate::fs::FileSystemDocumentStorage;
use crate::index::FileVectorIndex;
use crate::plan::FilePlanStore;
use crate::platform::{PlatformTrash, SystemTrash};
use crate::scan::FileSystemLibraryScanner;
use crate::state::{FileActivityLog, FileOperationJournal, FileSettingsRepository, FileTaskStore};
use crate::text::FileTextStore;
use crate::watch::FileSystemFolderWatcher;

/// Every storage adapter, with the state of this device in one app data directory. The composition root creates
/// one instance and hands the adapters to the use cases.
pub struct StorageAdapters {
    pub documents: Arc<FileSystemDocumentStorage>,
    pub catalogs: Arc<FileCatalogRepository>,
    pub scanner: Arc<FileSystemLibraryScanner>,
    pub texts: Arc<FileTextStore>,
    pub plans: Arc<FilePlanStore>,
    pub exports: Arc<FileExportWriter>,
    pub index: Arc<FileVectorIndex>,
    pub watcher: Arc<FileSystemFolderWatcher>,
    pub journal: Arc<FileOperationJournal>,
    pub tasks: Arc<FileTaskStore>,
    pub activity: Arc<FileActivityLog>,
    pub settings: Arc<FileSettingsRepository>,
}

impl StorageAdapters {
    /// The adapters with the system trash.
    #[must_use]
    pub fn new(app_data_directory: &Path) -> Self {
        Self::with_trash(app_data_directory, Arc::new(PlatformTrash))
    }

    #[must_use]
    pub fn with_trash(app_data_directory: &Path, trash: Arc<dyn SystemTrash>) -> Self {
        Self {
            documents: Arc::new(FileSystemDocumentStorage::new(Arc::clone(&trash))),
            catalogs: Arc::new(FileCatalogRepository::new(trash)),
            scanner: Arc::new(FileSystemLibraryScanner::new(app_data_directory.join("scan-cache"))),
            texts: Arc::new(FileTextStore),
            plans: Arc::new(FilePlanStore),
            exports: Arc::new(FileExportWriter),
            index: Arc::new(FileVectorIndex),
            watcher: Arc::new(FileSystemFolderWatcher),
            journal: Arc::new(FileOperationJournal::new(app_data_directory.join("journal.yaml"))),
            tasks: Arc::new(FileTaskStore::new(app_data_directory.join("tasks.yaml"))),
            activity: Arc::new(FileActivityLog::new(app_data_directory.join("activity.yaml"))),
            settings: Arc::new(FileSettingsRepository::new(app_data_directory.join("settings.yaml"))),
        }
    }
}
