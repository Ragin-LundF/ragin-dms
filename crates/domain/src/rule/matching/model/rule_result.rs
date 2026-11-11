use indexmap::{IndexMap, IndexSet};

use crate::field::{FieldKey, FieldValue};
use crate::library::RelativePath;
use crate::rule::{MatchingRuleId, RuleTarget};
use crate::taxonomy::{CategoryId, TagName};

/// What the matching rules propose for one document, with the rule behind each value so the review can say
/// "From rule: …". [`Self::correspondent`] is the one recognized in the text, if any. The [`Default`] is no
/// proposal at all.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuleResult {
    pub correspondent: Option<String>,
    pub category_id: Option<CategoryId>,
    pub tags: IndexSet<TagName>,
    pub fields: IndexMap<FieldKey, FieldValue>,
    pub target_folder: Option<RelativePath>,
    pub sources: IndexMap<RuleTarget, MatchingRuleId>,
}

impl RuleResult {
    /// `true` if the rules decided both category and correspondent, so the AI is not needed.
    #[must_use]
    pub fn is_sufficient(&self) -> bool {
        self.category_id.is_some() && self.correspondent.is_some()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.correspondent.is_none()
            && self.category_id.is_none()
            && self.tags.is_empty()
            && self.fields.is_empty()
            && self.target_folder.is_none()
    }
}
