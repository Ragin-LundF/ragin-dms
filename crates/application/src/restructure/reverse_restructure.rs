use std::sync::Arc;

use dms_domain::restructure::{PlannedMove, RestructurePlan};
use indexmap::IndexSet;

use crate::common::Progress;
use crate::library::CatalogStore;
use crate::port::plan::PlanStore;
use crate::restructure::{ApplyRestructurePlan, RestructureOutcome};

/// Undoes an applied restructure: every document that is still where the restructure put it goes back where it came
/// from, including files whose move was never catalogued. Documents moved or renamed since are left alone and
/// counted as skipped.
pub struct ReverseRestructure {
    store: Arc<CatalogStore>,
    plans: Arc<dyn PlanStore>,
    apply: Arc<ApplyRestructurePlan>,
}

impl ReverseRestructure {
    pub fn new(store: Arc<CatalogStore>, plans: Arc<dyn PlanStore>, apply: Arc<ApplyRestructurePlan>) -> Self {
        Self { store, plans, apply }
    }

    pub fn run(&self, name: &str, progress: &dyn Progress<(usize, usize)>) -> RestructureOutcome {
        let Some(library) = self.store.current() else {
            return RestructureOutcome::LibraryNotOpen;
        };
        let Some(record) = self.plans.read(library.folders.primary(), name) else {
            return RestructureOutcome::RecordNotFound;
        };
        // A document still catalogued at its old place moved on disk only; its file goes back as well.
        let (unchanged, changed): (Vec<&PlannedMove>, Vec<&PlannedMove>) = record.moves.iter().partition(|planned| {
            library
                .catalog
                .documents()
                .get(&planned.document_id)
                .is_some_and(|document| document.path == planned.to || document.path == planned.from)
        });
        let inverse = RestructurePlan {
            template: format!("{} (reversed)", record.template),
            moves: unchanged.iter().rev().map(|planned| planned.inverse()).collect(),
            unchanged: Vec::new(),
            skipped: Vec::new(),
            renamed_for_clash: IndexSet::new(),
        };
        self.apply.run(&inverse, changed.len(), progress)
    }
}
