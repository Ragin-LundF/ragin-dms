//! Wiring of the `inbox` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::inbox::{CollectInbox, ConfigureInbox, FileInboxDocument};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn configure_inbox(&self) -> ConfigureInbox {
        ConfigureInbox::new(self.store.clone(), self.folders.clone(), self.replicator.clone(), self.activity.clone())
    }

    pub fn collect_inbox(&self) -> CollectInbox {
        CollectInbox::new(self.store.clone(), Arc::new(self.adopt_new_file()))
    }

    pub fn file_inbox_document(&self) -> FileInboxDocument {
        FileInboxDocument::new(self.store.clone(), Arc::new(self.filing()), self.activity.clone())
    }
}
