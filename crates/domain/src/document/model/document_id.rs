use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies a document for its whole life. Paths change when a file is moved or renamed; the id does not.
    DocumentId,
    "Document id must not be blank"
);
