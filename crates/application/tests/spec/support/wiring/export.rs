//! Wiring of the `export` use cases on `TestLibrary`.

use dms_application::export::ExportDocuments;

use crate::support::TestLibrary;
use crate::support::fake::pdf::FakePdfEditor;
use std::sync::Arc;

impl TestLibrary {
    pub fn export_documents(&self) -> ExportDocuments {
        ExportDocuments::new(
            self.store.clone(),
            self.matcher.clone(),
            self.exports.clone(),
            Arc::new(FakePdfEditor::new(self.folders.clone())),
            self.activity.clone(),
            self.clock.clone(),
        )
    }
}
