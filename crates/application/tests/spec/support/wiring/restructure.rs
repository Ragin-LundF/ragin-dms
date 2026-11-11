//! Wiring of the `restructure` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::restructure::{
    ApplyRestructurePlan, ListRestructureHistory, PlanRestructure, RemoveEmptyFolders, ReverseRestructure,
};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn plan_restructure(&self) -> PlanRestructure {
        PlanRestructure::new(self.store.clone(), self.matcher.clone())
    }

    pub fn apply_restructure_plan(&self) -> ApplyRestructurePlan {
        ApplyRestructurePlan::new(
            self.store.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.plans.clone(),
            self.activity.clone(),
            self.clock.clone(),
        )
    }

    pub fn reverse_restructure(&self) -> ReverseRestructure {
        ReverseRestructure::new(self.store.clone(), self.plans.clone(), Arc::new(self.apply_restructure_plan()))
    }

    pub fn list_restructure_history(&self) -> ListRestructureHistory {
        ListRestructureHistory::new(self.store.clone(), self.plans.clone())
    }

    pub fn remove_empty_folders(&self) -> RemoveEmptyFolders {
        RemoveEmptyFolders::new(
            self.store.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.activity.clone(),
        )
    }
}
