//! Wiring of the `document` use cases on `TestLibrary`.

use dms_application::document::{EditDocumentDetails, RestoreDocument, TrashDuplicateFile, TrashNewFile};
use dms_application::port::storage::{LibraryScanner, NoScanListener};
use dms_domain::change::{ChangeDetector, DetectedChange};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn edit_document_details(&self) -> EditDocumentDetails {
        EditDocumentDetails::new(self.store.clone(), self.activity.clone())
    }

    pub fn restore_document(&self) -> RestoreDocument {
        RestoreDocument::new(self.store.clone(), self.folders.clone(), self.replicator.clone(), self.activity.clone())
    }

    pub fn trash_new_file(&self) -> TrashNewFile {
        TrashNewFile::new(self.store.clone(), self.folders.clone(), self.activity.clone())
    }

    pub fn trash_duplicate_file(&self) -> TrashDuplicateFile {
        TrashDuplicateFile::new(self.store.clone(), self.folders.clone(), self.activity.clone())
    }

    /// The changes a scan finds (Kotlin's `scanned()!!.changes`).
    #[track_caller]
    pub fn scan_changes(&self) -> Vec<DetectedChange> {
        self.scanned(false).expect("a library is open").changes
    }
}
