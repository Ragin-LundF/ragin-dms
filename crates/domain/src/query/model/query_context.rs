use chrono::NaiveDate;
use chrono_tz::Tz;

use crate::catalog::Catalog;

/// What a [`crate::query::DocumentQuery`] is evaluated against: the catalog (for the category tree), today (for
/// relative conditions such as "due within 30 days") and the zone that turns "added at" into a day.
#[derive(Debug, Clone, Copy)]
pub struct QueryContext<'a> {
    pub catalog: &'a Catalog,
    pub today: NaiveDate,
    pub zone: Tz,
}
