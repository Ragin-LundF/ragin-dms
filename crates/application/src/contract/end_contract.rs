use std::sync::Arc;

use chrono::NaiveDate;
use dms_domain::catalog::{CatalogRejection, CatalogUpdate};
use dms_domain::contract::{Contract, ContractId};

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Marks a contract as ended on a day (it stops counting in the totals after it and gets no cancel-by task), or,
/// with `None`, as running again.
pub struct EndContract {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
}

impl EndContract {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, activity }
    }

    pub fn run(&self, contract_id: &ContractId, ended_on: Option<NaiveDate>) -> OperationOutcome {
        let outcome = self.store.commit(|catalog| {
            let Some(contract) = catalog.contracts().get(contract_id) else {
                return Ok(CatalogUpdate::Rejected(CatalogRejection::ContractNotFound));
            };
            catalog.save_contract(Contract { ended_on, ..contract.clone() })
        });
        if let OperationOutcome::Done { catalog, .. } = &outcome
            && let Some(contract) = catalog.contracts().get(contract_id)
        {
            self.activity.record(ActivityKind::CatalogEdited, &contract.name);
        }
        outcome
    }
}
