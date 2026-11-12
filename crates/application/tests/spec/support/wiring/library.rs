//! Wiring of the `library` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::common::NoProgress;
use dms_application::library::{
    AddScannedFiles, BuildDashboard, ConfigureLibrary, Dashboard, DashboardOutcome, DetectDocumentDates, ScanLibrary,
    ScanOutcome, ScanProgress, ScanResult,
};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn configure_library(&self) -> ConfigureLibrary {
        ConfigureLibrary::new(self.settings.clone(), self.ids.clone())
    }

    pub fn add_scanned_files(&self) -> AddScannedFiles {
        AddScannedFiles::new(
            self.store.clone(),
            self.dates.clone(),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }

    pub fn detect_document_dates(&self) -> DetectDocumentDates {
        DetectDocumentDates::new(self.store.clone(), self.dates.clone(), self.activity.clone())
    }

    pub fn scan_library(&self) -> ScanLibrary {
        ScanLibrary::new(self.store.clone(), self.folders.clone(), Arc::new(self.add_scanned_files()))
    }

    pub fn build_dashboard(&self) -> BuildDashboard {
        BuildDashboard::new(
            self.store.clone(),
            self.tasks.clone(),
            self.journal.clone(),
            self.activity_log.clone(),
            self.clock.clone(),
            self.health.clone(),
        )
    }

    /// Scans like `scan_library`; `None` while no library is open.
    pub fn scanned(&self, add_new_files: bool) -> Option<ScanResult> {
        self.scanned_with(add_new_files, &NoProgress)
    }

    pub fn scanned_with(
        &self,
        add_new_files: bool,
        progress: &dyn dms_application::common::Progress<ScanProgress>,
    ) -> Option<ScanResult> {
        match self.scan_library().run(add_new_files, progress) {
            ScanOutcome::Scanned(result) => Some(result),
            _ => None,
        }
    }

    /// The dashboard `build_dashboard` builds; `None` while no library is open.
    pub fn dashboard(&self, scan: Option<&ScanResult>) -> Option<Dashboard> {
        match self.build_dashboard().run(scan) {
            DashboardOutcome::Built(dashboard) => Some(dashboard),
            DashboardOutcome::LibraryNotOpen => None,
        }
    }
}
