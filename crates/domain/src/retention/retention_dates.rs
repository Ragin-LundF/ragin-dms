use std::collections::HashSet;

use chrono::{Datelike, NaiveDate};

use crate::catalog::Catalog;
use crate::dates;
use crate::document::Document;
use crate::taxonomy::CategoryId;

/// When a document's retention ends. Nothing has a retention period unless the user set one: a keep-until date
/// on the document wins; otherwise the period of its category, or of the closest parent category that has one,
/// counts from the end of the document date's year.
pub struct RetentionDates;

impl RetentionDates {
    /// The date the retention ends, or `None` if the user set none or already marked it done.
    #[must_use]
    pub fn due_date(document: &Document, catalog: &Catalog) -> Option<NaiveDate> {
        let retention = document.retention.as_ref();
        if retention.is_some_and(|retention| retention.settled_at.is_some()) {
            return None;
        }
        if let Some(keep_until) = retention.and_then(|retention| retention.keep_until) {
            return Some(keep_until);
        }
        let years = category_years(document.category_id.as_ref(), catalog)?;
        let year = document.metadata.document_date.unwrap_or_else(|| document.added_at.date_naive()).year();
        Some(dates::last_day_of(i64::from(year) + i64::from(years)))
    }
}

fn category_years(category_id: Option<&CategoryId>, catalog: &Catalog) -> Option<u32> {
    let mut visited = HashSet::new();
    let mut current = category_id.and_then(|id| catalog.categories().get(id));
    while let Some(category) = current {
        if !visited.insert(&category.id) {
            break;
        }
        if let Some(years) = category.retention_years {
            return Some(years);
        }
        current = category.parent_id.as_ref().and_then(|id| catalog.categories().get(id));
    }
    None
}
