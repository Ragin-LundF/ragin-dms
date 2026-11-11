use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies a library. Every folder holding the same library carries the same id in its catalog.
    LibraryId,
    "Library id must not be blank"
);
