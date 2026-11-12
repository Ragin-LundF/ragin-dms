use chrono::NaiveDate;
use dms_domain::contract::{Notice, PricePoint};

/// What the AI read from a contract's documents, minus the price points the contract already has: each point is
/// accepted on its own, the terms as a whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractTermsSuggestion {
    pub prices: Vec<PricePoint>,
    pub notice: Option<Notice>,
    pub term_end: Option<NaiveDate>,
    pub renews_by_months: Option<u16>,
    pub model: String,
}
