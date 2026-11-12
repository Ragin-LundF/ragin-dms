use crate::contract::ContractTermsSuggestion;
use crate::port::ai::ClassificationFailure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractContractTermsOutcome {
    Suggested(ContractTermsSuggestion),
    LibraryNotOpen,
    NotFound,
    /// "Use AI" is off on this device.
    AiDisabled,
    /// None of the contract's documents has text the AI could read.
    NoText,
    Failed(ClassificationFailure),
}
