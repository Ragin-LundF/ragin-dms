use std::sync::Arc;

use crate::library::{BuildDashboard, CatalogStore, ConfigureLibrary, DetectDocumentDates, OpenLibrary, ScanLibrary};
use crate::port::state::SettingsRepository;
use crate::port::system::Clock;
use crate::preview::DocumentThumbnails;
use crate::search::{FindDocuments, SearchContents};

/// Opening, configuring, scanning and searching the library, and the catalog of the open library (`store`).
pub struct LibraryServices {
    pub store: Arc<CatalogStore>,
    pub settings: Arc<dyn SettingsRepository>,
    pub clock: Arc<dyn Clock>,
    pub configure_library: Arc<ConfigureLibrary>,
    pub open_library: Arc<OpenLibrary>,
    pub scan_library: Arc<ScanLibrary>,
    pub detect_document_dates: Arc<DetectDocumentDates>,
    pub build_dashboard: Arc<BuildDashboard>,
    pub search_contents: Arc<SearchContents>,
    pub find_documents: Arc<FindDocuments>,
    pub thumbnails: Arc<DocumentThumbnails>,
}
