use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies a case; a UUID.
    CaseId,
    "Case id must not be blank"
);
