use chrono::NaiveDate;

use crate::DomainError;
use crate::contract::ContractId;
use crate::document::DocumentId;
use crate::emergency::{EmergencyEntry, EmergencyItem};
use crate::error::{require, require_distinct, require_none_or_not_blank};

/// The curated selection of documents, contracts, contacts and notes for an emergency, refreshed every
/// `refresh_months`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmergencyFolder {
    pub items: Vec<EmergencyItem>,
    pub last_refreshed: Option<NaiveDate>,
    pub refresh_months: u8,
    pub intro_text: Option<String>,
}

impl Default for EmergencyFolder {
    fn default() -> Self {
        Self { items: Vec::new(), last_refreshed: None, refresh_months: Self::DEFAULT_REFRESH_MONTHS, intro_text: None }
    }
}

impl EmergencyFolder {
    pub const DEFAULT_REFRESH_MONTHS: u8 = 12;

    /// # Errors
    /// Fails for two items with the same id, a refresh every 0 months, a blank intro or an invalid entry.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_distinct(self.items.iter().map(|item| &item.id), || "Two emergency items share an id".to_owned())?;
        require(self.refresh_months > 0, || "The emergency folder is refreshed at least every month".to_owned())?;
        require_none_or_not_blank(self.intro_text.as_deref(), "Intro text")?;
        self.items.iter().try_for_each(|item| item.entry.validate())
    }

    pub fn referenced_documents(&self) -> impl Iterator<Item = &DocumentId> {
        self.items.iter().filter_map(|item| match &item.entry {
            EmergencyEntry::Document(id) => Some(id),
            EmergencyEntry::Contract(_) | EmergencyEntry::Contact { .. } | EmergencyEntry::Note(_) => None,
        })
    }

    pub fn referenced_contracts(&self) -> impl Iterator<Item = &ContractId> {
        self.items.iter().filter_map(|item| match &item.entry {
            EmergencyEntry::Contract(id) => Some(id),
            EmergencyEntry::Document(_) | EmergencyEntry::Contact { .. } | EmergencyEntry::Note(_) => None,
        })
    }

    /// The folder without the items of documents and contracts that `keep_document` or `keep_contract` refuse.
    #[must_use]
    pub fn retaining(
        &self,
        keep_document: &impl Fn(&DocumentId) -> bool,
        keep_contract: &impl Fn(&ContractId) -> bool,
    ) -> Self {
        let mut folder = self.clone();
        folder.items.retain(|item| match &item.entry {
            EmergencyEntry::Document(id) => keep_document(id),
            EmergencyEntry::Contract(id) => keep_contract(id),
            EmergencyEntry::Contact { .. } | EmergencyEntry::Note(_) => true,
        });
        folder
    }
}
