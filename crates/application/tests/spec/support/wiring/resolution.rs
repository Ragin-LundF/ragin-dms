//! Wiring of the `resolution` use cases on `TestLibrary`.

use dms_application::resolution::{DismissTask, KeepBothVersions, KeepVersion, MergeDuplicates, ResolveMove};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn keep_version(&self) -> KeepVersion {
        KeepVersion::new(self.store.clone(), self.folders.clone(), self.replicator.clone(), self.activity.clone())
    }

    pub fn keep_both_versions(&self) -> KeepBothVersions {
        KeepBothVersions::new(
            self.store.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.names.clone(),
            self.activity.clone(),
        )
    }

    pub fn resolve_move(&self) -> ResolveMove {
        ResolveMove::new(self.store.clone(), self.folders.clone(), self.replicator.clone(), self.activity.clone())
    }

    pub fn dismiss_task(&self) -> DismissTask {
        DismissTask::new(self.tasks.clone())
    }

    pub fn merge_duplicates(&self) -> MergeDuplicates {
        MergeDuplicates::new(self.store.clone(), self.folders.clone(), self.tasks.clone(), self.activity.clone())
    }
}
