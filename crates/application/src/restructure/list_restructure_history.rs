use std::sync::Arc;

use crate::library::CatalogStore;
use crate::port::plan::PlanStore;
use crate::restructure::RestructureHistoryOutcome;

/// The applied restructures that can be reversed, newest first.
pub struct ListRestructureHistory {
    store: Arc<CatalogStore>,
    plans: Arc<dyn PlanStore>,
}

impl ListRestructureHistory {
    pub fn new(store: Arc<CatalogStore>, plans: Arc<dyn PlanStore>) -> Self {
        Self { store, plans }
    }

    pub fn run(&self) -> RestructureHistoryOutcome {
        let Some(library) = self.store.current() else {
            return RestructureHistoryOutcome::LibraryNotOpen;
        };
        RestructureHistoryOutcome::Listed(self.plans.names(library.folders.primary()))
    }
}
