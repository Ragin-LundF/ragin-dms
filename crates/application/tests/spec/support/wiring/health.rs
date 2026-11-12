//! Wiring of the `health` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::health::{CompareCatalogSnapshot, RepairCopy, RestoreCatalogSnapshot, VerifyLibrary};

use crate::support::TestLibrary;
use crate::support::fake::pdf::FakePdfEditor;

impl TestLibrary {
    pub fn verify_library(&self) -> VerifyLibrary {
        VerifyLibrary::new(
            self.store.clone(),
            self.folders.clone(),
            Arc::new(FakePdfEditor::new(self.folders.clone())),
            self.health.clone(),
            self.clock.clone(),
        )
    }

    pub fn repair_copy(&self) -> RepairCopy {
        RepairCopy::new(
            self.store.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.health.clone(),
            self.activity.clone(),
        )
    }

    pub fn compare_catalog_snapshot(&self) -> CompareCatalogSnapshot {
        CompareCatalogSnapshot::new(self.store.clone(), self.snapshots.clone())
    }

    pub fn restore_catalog_snapshot(&self) -> RestoreCatalogSnapshot {
        RestoreCatalogSnapshot::new(self.store.clone(), self.snapshots.clone(), self.activity.clone())
    }
}
