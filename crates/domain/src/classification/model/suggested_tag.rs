use crate::taxonomy::TagName;

/// A suggested tag; [`Self::is_new`] marks tags the catalog does not know yet, so the UI can highlight them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SuggestedTag {
    pub name: TagName,
    pub is_new: bool,
}
