use std::sync::Arc;

use dms_domain::catalog::{CatalogRejection, CatalogUpdate};

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::contract::AttachDocumentsInput;
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Attaches documents to a contract, or takes them off it; the documents themselves stay as they are.
pub struct AttachDocumentsToContract {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
}

impl AttachDocumentsToContract {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, activity }
    }

    pub fn run(&self, input: &AttachDocumentsInput) -> OperationOutcome {
        let outcome = self.store.commit(|catalog| {
            let Some(contract) = catalog.contracts().get(&input.contract_id) else {
                return Ok(CatalogUpdate::Rejected(CatalogRejection::ContractNotFound));
            };
            let mut contract = contract.clone();
            if input.detach {
                contract.document_ids.retain(|id| !input.document_ids.contains(id));
            } else {
                contract.document_ids.extend(input.document_ids.iter().cloned());
            }
            catalog.save_contract(contract)
        });
        if let OperationOutcome::Done { catalog, .. } = &outcome
            && let Some(contract) = catalog.contracts().get(&input.contract_id)
        {
            self.activity.record(ActivityKind::CatalogEdited, &contract.name);
        }
        outcome
    }
}
