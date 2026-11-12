use chrono::NaiveDate;

use super::tax_assignment::validate_tax_year;
use crate::DomainError;
use crate::document::DocumentId;
use crate::error::require_distinct;
use crate::tax::TaxChecklistItem;

/// The checklist of one tax year and the day the return was sent, once it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxYear {
    pub year: i32,
    pub items: Vec<TaxChecklistItem>,
    pub sent_on: Option<NaiveDate>,
}

impl TaxYear {
    #[must_use]
    pub fn new(year: i32) -> Self {
        Self { year, items: Vec::new(), sent_on: None }
    }

    /// # Errors
    /// Fails for a year outside 1000..=9999, two items with the same id, or an invalid item.
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_tax_year(self.year)?;
        require_distinct(self.items.iter().map(|item| &item.id), || "Two tax items share an id".to_owned())?;
        self.items.iter().try_for_each(TaxChecklistItem::validate)
    }

    pub fn referenced_documents(&self) -> impl Iterator<Item = &DocumentId> {
        self.items.iter().filter_map(|item| item.document_id.as_ref())
    }

    /// The year without the documents `keep` refuses; their items stay, without a document.
    #[must_use]
    pub fn retaining_documents(&self, keep: &impl Fn(&DocumentId) -> bool) -> Self {
        let mut year = self.clone();
        for item in &mut year.items {
            item.document_id = item.document_id.take().filter(|id| keep(id));
        }
        year
    }
}
