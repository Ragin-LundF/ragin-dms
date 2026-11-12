use dms_domain::contract::ContractId;
use dms_domain::document::DocumentId;

/// Documents to attach to a contract, or with `detach` to take off it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachDocumentsInput {
    pub contract_id: ContractId,
    pub document_ids: Vec<DocumentId>,
    pub detach: bool,
}
