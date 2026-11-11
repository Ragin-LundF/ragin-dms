use chrono::{Datelike, NaiveDate};
use dms_domain::field::{Decimal, FieldPresets, FieldValue};
use dms_domain::query::{
    DateRange, DocumentQuery, FieldComparison, FieldCondition, SavedSearch, SavedSearchId, TagFilter,
};
use dms_domain::taxonomy::TagName;
use indexmap::IndexSet;

use crate::port::system::IdGenerator;

const DUE_DAYS: &str = "30";

/// The smart folders a library starts with: "Untagged", "Uncategorized", the tax documents of the current year,
/// and what is due in the next 30 days.
pub struct SavedSearchPresets;

impl SavedSearchPresets {
    pub fn of(today: NaiveDate, ids: &dyn IdGenerator, tax_tag: &TagName) -> Vec<SavedSearch> {
        let due = FieldCondition {
            key: FieldPresets::due_date().key,
            comparison: FieldComparison::WithinDays,
            value: Some(FieldValue::Number(Decimal::parse(DUE_DAYS).expect("invariant: a number"))),
        };
        let presets = [
            ("Untagged".to_owned(), DocumentQuery { untagged: true, ..DocumentQuery::default() }),
            ("Uncategorized".to_owned(), DocumentQuery { uncategorized: true, ..DocumentQuery::default() }),
            (
                format!("Tax {}", today.year()),
                DocumentQuery {
                    tags: TagFilter { all: IndexSet::from([tax_tag.clone()]), ..TagFilter::default() },
                    dates: Some(DateRange::year(today.year()).expect("invariant: a calendar year")),
                    ..DocumentQuery::default()
                },
            ),
            ("Due in 30 days".to_owned(), DocumentQuery { fields: vec![due], ..DocumentQuery::default() }),
        ];
        presets
            .into_iter()
            .map(|(name, query)| SavedSearch {
                id: SavedSearchId::new(ids.new_id()).expect("invariant: generated ids are not blank"),
                name,
                query,
            })
            .collect()
    }
}
