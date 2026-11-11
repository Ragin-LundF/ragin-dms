use std::sync::Arc;

use dms_domain::query::DocumentQuery;

use crate::library::CatalogStore;
use crate::search::{ContentMatcher, FindOutcome};

/// Finds the documents a query matches, its content term included; see [`ContentMatcher`]. Never writes.
pub struct FindDocuments {
    store: Arc<CatalogStore>,
    matcher: Arc<ContentMatcher>,
}

impl FindDocuments {
    pub fn new(store: Arc<CatalogStore>, matcher: Arc<ContentMatcher>) -> Self {
        Self { store, matcher }
    }

    pub fn run(&self, query: &DocumentQuery) -> FindOutcome {
        let Some(library) = self.store.current() else {
            return FindOutcome::LibraryNotOpen;
        };
        FindOutcome::Found(self.matcher.matching(&library, query))
    }
}
