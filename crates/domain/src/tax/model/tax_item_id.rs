use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies an item of a tax year's checklist; a UUID.
    TaxItemId,
    "Tax item id must not be blank"
);
