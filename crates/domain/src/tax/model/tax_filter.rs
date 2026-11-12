use chrono::NaiveDate;

use crate::DomainError;
use crate::document::Document;
use crate::query::DateRange;
use crate::tax::TaxArea;
use crate::taxonomy::TagName;

use super::tax_assignment::validate_tax_year;

/// The documents assigned to a tax year, an area, or both. A document without a tax assignment matches only
/// through [`Self::tag_fallback`]: it carries that tag and, with a [`Self::year`], is dated in that year. This keeps
/// the tax documents that were tagged before tax years existed in the ready-made smart folder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaxFilter {
    pub year: Option<i32>,
    pub area: Option<TaxArea>,
    pub tag_fallback: Option<TagName>,
}

impl TaxFilter {
    /// # Errors
    /// Fails for a year outside 1000..=9999.
    pub fn validate(&self) -> Result<(), DomainError> {
        self.year.map_or(Ok(()), validate_tax_year)
    }

    /// `date` is the day the query counts for the document (its document date, or the day it was added).
    #[must_use]
    pub fn matches(&self, document: &Document, date: NaiveDate) -> bool {
        match &document.tax {
            Some(tax) => self.year.is_none_or(|year| tax.year == year) && self.area.is_none_or(|area| tax.area == area),
            None => {
                self.area.is_none()
                    && self.tag_fallback.as_ref().is_some_and(|tag| document.tags.contains(tag))
                    && self.year.is_none_or(|year| DateRange::year(year).is_ok_and(|range| range.contains(date)))
            }
        }
    }
}
