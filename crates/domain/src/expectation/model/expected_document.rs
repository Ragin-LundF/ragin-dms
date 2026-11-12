use chrono::NaiveDate;

use crate::expectation::Rhythm;
use crate::taxonomy::CategoryId;

/// The next document a sender is expected to send, of one category (or none), learned from the dates of its
/// earlier documents; `first_seen_year` is the year of the first of them ("every January since 2021").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpectedDocument {
    pub correspondent: String,
    pub category_id: Option<CategoryId>,
    pub expected_on: NaiveDate,
    pub rhythm: Rhythm,
    pub first_seen_year: i32,
}
