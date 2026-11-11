use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies a category independently of its (renamable) name.
    CategoryId,
    "Category id must not be blank"
);
