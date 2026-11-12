use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies an item of the emergency folder; a UUID.
    EmergencyItemId,
    "Emergency item id must not be blank"
);
