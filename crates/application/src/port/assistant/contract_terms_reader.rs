use crate::port::assistant::{ContractTermsAnswer, ContractTermsRequest};

/// Lets the AI read a contract's prices and terms from its documents. The answer is only a suggestion the user
/// accepts point by point.
pub trait ContractTermsReader: Send + Sync {
    fn read(&self, request: &ContractTermsRequest) -> ContractTermsAnswer;
}
