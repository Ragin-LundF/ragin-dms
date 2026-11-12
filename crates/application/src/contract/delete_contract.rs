use std::sync::Arc;

use dms_domain::contract::ContractId;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Removes a contract from the catalog after the user confirmed it; its documents stay in the library.
pub struct DeleteContract {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
}

impl DeleteContract {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, activity }
    }

    pub fn run(&self, contract_id: &ContractId, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let name = self.store.with_catalog(|catalog| {
            catalog.and_then(|catalog| catalog.contracts().get(contract_id)).map(|contract| contract.name.clone())
        });
        let outcome = self.store.commit(|catalog| Ok(catalog.delete_contract(contract_id)));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::CatalogEdited, &name.unwrap_or_default());
        }
        outcome
    }
}
