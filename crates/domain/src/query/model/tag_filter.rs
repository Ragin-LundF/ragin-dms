use indexmap::IndexSet;

use crate::taxonomy::TagName;

/// Tag conditions: a document needs every tag of [`Self::all`], at least one of [`Self::any`], and none of
/// [`Self::none`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TagFilter {
    pub all: IndexSet<TagName>,
    pub any: IndexSet<TagName>,
    pub none: IndexSet<TagName>,
}

impl TagFilter {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.all.is_empty() && self.any.is_empty() && self.none.is_empty()
    }

    #[must_use]
    pub fn matches(&self, tags: &IndexSet<TagName>) -> bool {
        self.all.is_subset(tags)
            && (self.any.is_empty() || self.any.iter().any(|tag| tags.contains(tag)))
            && !self.none.iter().any(|tag| tags.contains(tag))
    }
}
