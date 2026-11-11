use std::cmp::{Ordering, Reverse};

use dms_domain::document::Document;

/// Which of several copies of a file to keep by default: the one the user did the most work on (category, tags, AI
/// classification, title), and among equals the one that was catalogued first.
pub struct DuplicatePreference;

impl DuplicatePreference {
    /// The preferred copy; `None` for no copies (Kotlin threw `NoSuchElementException`).
    #[must_use]
    pub fn preferred(copies: &[Document]) -> Option<&Document> {
        // `min_by` returns the first of equal elements, like Kotlin's stable `sortedWith(..).first()`.
        copies.iter().min_by(|first, second| compare(first, second))
    }
}

fn compare(first: &Document, second: &Document) -> Ordering {
    let key = |document: &Document| {
        (
            Reverse(document.category_id.is_some()),
            Reverse(document.tags.len()),
            Reverse(document.classification.is_some()),
            Reverse(document.metadata.title.is_some()),
            document.added_at,
        )
    };
    key(first).cmp(&key(second))
}
