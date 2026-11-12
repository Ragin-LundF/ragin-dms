/// Why the app proposes to record a correspondent's documents as a contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProposalReason {
    /// A document has a cancel-by or contract-end date.
    DeadlineField,
    /// Several documents are in a category named like contract, insurance or subscription.
    ContractCategory,
    /// The correspondent sends bills in a regular rhythm.
    RecurringBills,
}
