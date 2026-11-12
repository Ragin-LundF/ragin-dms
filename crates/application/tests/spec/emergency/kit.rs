use std::ops::Deref;
use std::sync::Arc;

use dms_application::emergency::{
    CreateEmergencyOverview, EditEmergencyFolder, EmergencyEdit, ExportEmergencyFolder, MarkEmergencyFolderRefreshed,
};
use dms_domain::emergency::{EmergencyEntry, EmergencyFolder, EmergencyGroup};

use crate::support::TestLibrary;
use crate::support::fake::pdf::FakePdfEditor;
use crate::support::test_data::done;

/// An open `TestLibrary` with the PDF editor fake and the emergency use cases.
pub struct EmergencyKit {
    pub library: TestLibrary,
    pub pdf: Arc<FakePdfEditor>,
}

impl Deref for EmergencyKit {
    type Target = TestLibrary;

    fn deref(&self) -> &TestLibrary {
        &self.library
    }
}

impl EmergencyKit {
    pub fn new() -> Self {
        let library = TestLibrary::new();
        library.open();
        Self { pdf: Arc::new(FakePdfEditor::new(library.folders.clone())), library }
    }

    pub fn edit(&self) -> EditEmergencyFolder {
        EditEmergencyFolder::new(self.store.clone(), Arc::new(self.change_settings()), self.ids.clone())
    }

    pub fn mark_refreshed(&self) -> MarkEmergencyFolderRefreshed {
        MarkEmergencyFolderRefreshed::new(self.store.clone(), Arc::new(self.change_settings()), self.clock.clone())
    }

    pub fn create_overview(&self) -> CreateEmergencyOverview {
        CreateEmergencyOverview::new(
            self.store.clone(),
            self.pdf.clone(),
            self.exports.clone(),
            self.replicator.clone(),
            self.names.clone(),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }

    pub fn export(&self) -> ExportEmergencyFolder {
        ExportEmergencyFolder::new(
            self.store.clone(),
            self.pdf.clone(),
            self.exports.clone(),
            self.activity.clone(),
            self.clock.clone(),
        )
    }

    pub fn add_item(&self, group: EmergencyGroup, entry: EmergencyEntry) {
        done(self.edit().run(&EmergencyEdit::Add { group, entry }));
    }

    pub fn folder(&self) -> EmergencyFolder {
        self.catalog().settings().emergency_folder.clone().expect("an emergency folder")
    }
}
