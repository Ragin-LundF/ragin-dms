use indexmap::IndexSet;

use crate::DomainError;
use crate::error::require;
use crate::taxonomy::{CategoryId, TagName};
use crate::text;

/// A rule the library suggests after the user approved enough documents of one [`Self::correspondent`] the same
/// way: [`Self::document_count`] documents, most of them with [`Self::category_id`] and [`Self::tags`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleProposal {
    pub correspondent: String,
    pub category_id: Option<CategoryId>,
    pub tags: IndexSet<TagName>,
    pub document_count: usize,
}

impl RuleProposal {
    /// # Errors
    /// Fails like [`Self::validate`].
    pub fn new(
        correspondent: impl Into<String>,
        category_id: Option<CategoryId>,
        tags: IndexSet<TagName>,
        document_count: usize,
    ) -> Result<Self, DomainError> {
        let proposal = Self { correspondent: correspondent.into(), category_id, tags, document_count };
        proposal.validate()?;
        Ok(proposal)
    }

    /// # Errors
    /// Fails for a blank correspondent or a proposal without category and tags.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!text::is_blank(&self.correspondent), || "Correspondent must not be blank".to_owned())?;
        require(self.category_id.is_some() || !self.tags.is_empty(), || {
            "A proposal needs a category or tags".to_owned()
        })
    }

    /// Identifies what is proposed, so a rejected proposal is not proposed again.
    #[must_use]
    pub fn signature(&self) -> String {
        let mut tags: Vec<&TagName> = self.tags.iter().collect();
        tags.sort();
        let tag_list: Vec<String> = tags.iter().map(|tag| text::lowercase(tag.value())).collect();
        let category = self.category_id.as_ref().map_or("", |id| id.value());
        format!("{}|{category}|{}", text::lowercase(text::trim(&self.correspondent)), tag_list.join(","))
    }
}
