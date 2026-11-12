use crate::port::ai::ClassificationFailure;
use crate::port::assistant::ContractTermsReading;

/// The AI's reading of a contract's terms, from `model`, or why there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractTermsAnswer {
    Read { reading: ContractTermsReading, model: String },
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
