use chrono::NaiveDate;
use dms_domain::contract::{ContractPrice, Notice};

/// What the AI read from a contract's documents; every part may be missing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContractTermsReading {
    pub prices: Vec<ReadPrice>,
    pub notice: Option<Notice>,
    pub term_end: Option<NaiveDate>,
    pub renews_by_months: Option<u16>,
}

/// A price valid from a day on, with the number (from 1) of the source text that names it, if the AI gave one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadPrice {
    pub valid_from: NaiveDate,
    pub price: ContractPrice,
    pub source: Option<usize>,
}
