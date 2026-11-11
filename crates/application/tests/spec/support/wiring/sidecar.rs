//! Wiring of the `sidecar` use cases on `TestLibrary`.

use dms_application::sidecar::MigrateSidecarLayout;

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn migrate_sidecar_layout(&self) -> MigrateSidecarLayout {
        MigrateSidecarLayout::new(self.store.clone(), self.sidecar_layout.clone())
    }
}
