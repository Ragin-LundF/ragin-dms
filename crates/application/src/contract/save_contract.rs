use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::contract::{Contract, ContractId, PricePoint};

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::contract::{ContractDraft, SaveContractOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::system::{Clock, IdGenerator};

/// Records a new contract or saves the editor's changes to one, keeping its price history, documents and end date.
pub struct SaveContract {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl SaveContract {
    pub fn new(
        store: Arc<CatalogStore>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, activity, clock, ids }
    }

    pub fn run(&self, draft: &ContractDraft) -> SaveContractOutcome {
        let contract_id = draft
            .id
            .clone()
            .unwrap_or_else(|| ContractId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"));
        let outcome = self.store.commit(|catalog| self.saved(catalog, &contract_id, draft));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::CatalogEdited, &draft.name);
        }
        SaveContractOutcome { contract_id, outcome }
    }

    fn saved(&self, catalog: &Catalog, id: &ContractId, draft: &ContractDraft) -> Result<CatalogUpdate, DomainError> {
        let existing = catalog.contracts().get(id);
        if draft.id.is_some() && existing.is_none() {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::ContractNotFound));
        }
        let mut contract = match existing {
            Some(contract) => contract.clone(),
            None => Contract::new(id.clone(), draft.name.clone(), draft.kind)?,
        };
        contract.name.clone_from(&draft.name);
        contract.correspondent.clone_from(&draft.correspondent);
        contract.kind = draft.kind;
        contract.terms = draft.terms.clone();
        contract.note.clone_from(&draft.note);
        contract.document_ids.extend(draft.document_ids.iter().cloned());
        if draft.price != contract.price {
            contract = match (&draft.price, draft.price_valid_from) {
                (Some(price), Some(valid_from)) => contract.with_price_point(
                    PricePoint { valid_from, price: price.clone(), document_id: None },
                    self.clock.today(),
                ),
                (price, _) => Contract { price: price.clone(), ..contract },
            };
        }
        let today = self.clock.today();
        for point in &draft.added_prices {
            contract = contract.with_price_point(point.clone(), today);
        }
        catalog.save_contract(contract)
    }
}
