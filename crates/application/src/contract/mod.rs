//! Contracts and subscriptions: recording, ending, deleting and attaching documents, dismissing a proposal, and the
//! AI's reading of prices and terms, which is only a suggestion.

mod attach_documents_to_contract;
mod delete_contract;
mod dismiss_contract_proposal;
mod end_contract;
mod extract_contract_terms;
pub mod model;
mod save_contract;

pub use attach_documents_to_contract::AttachDocumentsToContract;
pub use delete_contract::DeleteContract;
pub use dismiss_contract_proposal::DismissContractProposal;
pub use end_contract::EndContract;
pub use extract_contract_terms::ExtractContractTerms;
pub use model::{
    AttachDocumentsInput, ContractDraft, ContractTermsSuggestion, ExtractContractTermsOutcome, SaveContractOutcome,
};
pub use save_contract::SaveContract;
