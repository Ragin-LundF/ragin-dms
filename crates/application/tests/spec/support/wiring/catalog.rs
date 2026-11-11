//! Wiring of the `catalog` use cases on `TestLibrary`.

use dms_application::catalog::{ChangeSettings, EditCatalog};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn change_settings(&self) -> ChangeSettings {
        ChangeSettings::new(self.store.clone(), self.activity.clone())
    }

    pub fn edit_catalog(&self) -> EditCatalog {
        EditCatalog::new(self.store.clone(), self.activity.clone(), self.ids.clone())
    }
}
