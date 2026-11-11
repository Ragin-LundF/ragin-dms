use std::sync::Arc;

use crate::library::CatalogStore;
use crate::search::{ContentMatcher, SearchOutcome};

/// Finds the documents whose text contains a query, ignoring case; see [`ContentMatcher`]. Never writes.
pub struct SearchContents {
    store: Arc<CatalogStore>,
    matcher: Arc<ContentMatcher>,
}

impl SearchContents {
    pub fn new(store: Arc<CatalogStore>, matcher: Arc<ContentMatcher>) -> Self {
        Self { store, matcher }
    }

    pub fn run(&self, query: &str) -> SearchOutcome {
        let Some(library) = self.store.current() else {
            return SearchOutcome::LibraryNotOpen;
        };
        SearchOutcome::Found(self.matcher.containing(&library, query))
    }
}
