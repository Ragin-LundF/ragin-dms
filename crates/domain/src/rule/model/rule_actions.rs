use indexmap::{IndexMap, IndexSet};

use crate::field::{FieldKey, FieldValue};
use crate::library::RelativePath;
use crate::taxonomy::{CategoryId, TagName};

/// What a matching rule proposes for a document. Proposals only: the user approves them in the review.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuleActions {
    pub category_id: Option<CategoryId>,
    pub tags: IndexSet<TagName>,
    pub fields: IndexMap<FieldKey, FieldValue>,
    pub target_folder: Option<RelativePath>,
}

impl RuleActions {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.category_id.is_none() && self.tags.is_empty() && self.fields.is_empty() && self.target_folder.is_none()
    }
}
