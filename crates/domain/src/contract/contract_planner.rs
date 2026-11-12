use std::collections::HashSet;

use chrono::NaiveDate;

use crate::catalog::Catalog;
use crate::contract::{Contract, ContractProposer};
use crate::dates;
use crate::document::DocumentId;
use crate::task::OpenTask;

/// The dashboard tasks of contracts: proposals to record one, and a running contract's next cancel-by date from
/// [`Self::LEAD_DAYS`] ahead until it passed (then the next term's date follows).
pub struct ContractPlanner;

impl ContractPlanner {
    pub const LEAD_DAYS: i64 = 30;

    #[must_use]
    pub fn due_tasks(catalog: &Catalog, today: NaiveDate) -> Vec<OpenTask> {
        let proposals = ContractProposer::proposals(catalog, today).into_iter().map(OpenTask::ReviewContractProposal);
        let cancel_by = catalog.contracts().values().filter_map(|contract| {
            let date = planned_cancel_by(contract, today)?;
            (date <= dates::plus_days(today, Self::LEAD_DAYS))
                .then(|| OpenTask::ContractCancelBy { contract_id: contract.id.clone(), date })
        });
        proposals.chain(cancel_by).collect()
    }

    /// The documents of running contracts that plan their own cancel-by date; their cancel-by fields need no task
    /// of their own.
    #[must_use]
    pub fn documents_with_planned_cancel_by(catalog: &Catalog, today: NaiveDate) -> HashSet<&DocumentId> {
        catalog
            .contracts()
            .values()
            .filter(|contract| planned_cancel_by(contract, today).is_some())
            .flat_map(|contract| contract.document_ids.iter())
            .collect()
    }
}

/// The next cancel-by date of a contract nobody ended yet; an ended contract (even one ending later) is cancelled.
fn planned_cancel_by(contract: &Contract, today: NaiveDate) -> Option<NaiveDate> {
    if contract.ended_on.is_some() {
        return None;
    }
    contract.terms.next_cancel_by(today)
}
