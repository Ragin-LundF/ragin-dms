use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies a follow-up of a case; a UUID.
    FollowUpId,
    "Follow-up id must not be blank"
);
