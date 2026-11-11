use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies a saved search independently of its (renamable) name.
    SavedSearchId,
    "Saved search id must not be blank"
);
