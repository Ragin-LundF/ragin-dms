use dms_domain::contract::ContractId;

use crate::common::OperationOutcome;

/// The id the saved contract has (a new one for a new contract) and how the save went.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveContractOutcome {
    pub contract_id: ContractId,
    pub outcome: OperationOutcome,
}
