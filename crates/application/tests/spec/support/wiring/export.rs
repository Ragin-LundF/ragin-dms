//! Wiring of the `export` use cases on `TestLibrary`.

use dms_application::export::ExportDocuments;

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn export_documents(&self) -> ExportDocuments {
        ExportDocuments::new(
            self.store.clone(),
            self.matcher.clone(),
            self.exports.clone(),
            self.activity.clone(),
            self.clock.clone(),
        )
    }
}
