//! Configuring library folders, opening and scanning the library, the dashboard, and `CatalogStore`, the only
//! writer of the catalog.

mod add_scanned_files;
mod build_dashboard;
mod catalog_store;
mod configure_library;
mod detect_document_dates;
pub mod model;
mod open_library;
mod scan_library;

pub use add_scanned_files::AddScannedFiles;
pub use build_dashboard::BuildDashboard;
pub use catalog_store::CatalogStore;
pub use configure_library::ConfigureLibrary;
pub use detect_document_dates::DetectDocumentDates;
pub use model::{
    ConfigureLibraryOutcome, Dashboard, DashboardOutcome, FolderState, LibraryCommand, OpenLibraryOutcome, ScanOutcome,
    ScanProgress, ScanResult, ScannedFile,
};
pub use open_library::OpenLibrary;
pub use scan_library::ScanLibrary;
