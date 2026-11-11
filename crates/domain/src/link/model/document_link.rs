use std::cmp::Ordering;

use crate::DomainError;
use crate::document::DocumentId;
use crate::error::require;
use crate::text;

/// Two documents that belong together, such as an invoice and its reminder. A link has no direction:
/// [`Self::first`] is always the smaller id, so the same pair is the same link however it was created.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentLink {
    first: DocumentId,
    second: DocumentId,
    label: Option<String>,
}

impl DocumentLink {
    /// The label is trimmed.
    ///
    /// # Errors
    /// Fails for a link of a document to itself or a blank label.
    pub fn new(first: DocumentId, second: DocumentId, label: Option<&str>) -> Result<Self, DomainError> {
        require(first != second, || "A document cannot be linked to itself".to_owned())?;
        require(label.is_none_or(|label| !text::is_blank(label)), || {
            "Link label must be null or not blank".to_owned()
        })?;
        let (first, second) = if text::compare(first.value(), second.value()) == Ordering::Greater {
            (second, first)
        } else {
            (first, second)
        };
        Ok(Self { first, second, label: label.map(|label| text::trim(label).to_owned()) })
    }

    #[must_use]
    pub fn first(&self) -> &DocumentId {
        &self.first
    }

    #[must_use]
    pub fn second(&self) -> &DocumentId {
        &self.second
    }

    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// `true` if this link connects `document_id` with another document.
    #[must_use]
    pub fn touches(&self, document_id: &DocumentId) -> bool {
        &self.first == document_id || &self.second == document_id
    }

    /// The other end of the link seen from `document_id`.
    #[must_use]
    pub fn other_than(&self, document_id: &DocumentId) -> &DocumentId {
        if &self.first == document_id { &self.second } else { &self.first }
    }

    /// `true` if both links connect the same two documents, whatever their labels.
    #[must_use]
    pub fn same_pair(&self, other: &Self) -> bool {
        self.first == other.first && self.second == other.second
    }
}
