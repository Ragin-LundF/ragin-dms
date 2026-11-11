//! The library folders on disk: files, the catalog file, scanning, the text store and the journal.

mod catalog_repository;
mod document_storage;
mod folder_watcher;
mod library_scanner;
pub mod model;
mod operation_journal;
mod scan_listener;
mod text_store;

pub use catalog_repository::CatalogRepository;
pub use document_storage::DocumentStorage;
pub use folder_watcher::FolderWatcher;
pub use library_scanner::LibraryScanner;
pub use model::{
    CatalogLoad, ExternalFile, FolderEntries, FolderOperation, PendingOperation, StorageError, StorageFailure,
    StorageResult,
};
pub use operation_journal::OperationJournal;
pub use scan_listener::{NoScanListener, ScanListener};
pub use text_store::TextStore;
