use crate::contract::ProposalReason;
use crate::document::DocumentId;

/// A correspondent whose documents look like a contract the user has not recorded yet.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContractProposal {
    pub correspondent: String,
    pub document_ids: Vec<DocumentId>,
    pub reason: ProposalReason,
}
