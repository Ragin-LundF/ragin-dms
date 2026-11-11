use std::sync::Arc;

use chrono::NaiveDate;
use dms_domain::document::{DocumentMetadata, FileNameDates};
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::port::storage::DocumentStorage;

/// The date of a document the user has not given one: the date in its file name, otherwise the day the file was
/// created. Without either the document has no date, and the day it was added stands in for it wherever a date
/// is shown. The user can change it at any time.
pub struct DocumentDates {
    storage: Arc<dyn DocumentStorage>,
}

impl DocumentDates {
    pub fn new(storage: Arc<dyn DocumentStorage>) -> Self {
        Self { storage }
    }

    pub fn guess(&self, folder: &LibraryFolder, path: &RelativePath) -> Option<NaiveDate> {
        FileNameDates::date_in(path.file_name()).or_else(|| self.storage.created_on(folder, path))
    }

    /// Metadata with the guessed date, for a document that is about to be catalogued.
    pub fn metadata_for(&self, folder: &LibraryFolder, path: &RelativePath) -> DocumentMetadata {
        DocumentMetadata { document_date: self.guess(folder, path), ..DocumentMetadata::default() }
    }
}
