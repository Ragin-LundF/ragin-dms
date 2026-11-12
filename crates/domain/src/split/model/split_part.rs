use chrono::NaiveDate;

use crate::classification::SuggestedCategory;

/// One document of a stack: its pages, counted from 0, both included, and what the AI read about it. Every value is
/// a suggestion for the review after the split.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitPart {
    pub first_page: usize,
    pub last_page: usize,
    pub title: Option<String>,
    pub category: Option<SuggestedCategory>,
    pub correspondent: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub confidence: Option<f64>,
}

impl SplitPart {
    /// The pages from `first_page` to `last_page` without anything read about them.
    #[must_use]
    pub fn pages(first_page: usize, last_page: usize) -> Self {
        Self {
            first_page,
            last_page,
            title: None,
            category: None,
            correspondent: None,
            document_date: None,
            confidence: None,
        }
    }

    /// `true` if the AI said anything about this part beyond its pages.
    #[must_use]
    pub fn has_suggestion(&self) -> bool {
        self.title.is_some() || self.category.is_some() || self.correspondent.is_some() || self.document_date.is_some()
    }
}
