//! The words of a split request: every page with its number, blank pages marked, and the library's categories.

use std::fmt::Write;

use dms_application::port::assistant::SplitRequest;
use dms_domain::text;

use crate::classify::prompt_texts::{listed, path_of};
use crate::prompt;

pub(crate) fn request(request: &SplitRequest) -> String {
    let mut categories: Vec<String> =
        request.categories.iter().map(|category| path_of(category, &request.categories)).collect();
    categories.sort_by(|first, second| text::compare(first, second));
    let mut pages = String::new();
    for (index, page) in request.pages.iter().enumerate() {
        let blank = if page.blank { " [blank]" } else { "" };
        // Writing to a String cannot fail.
        let _ = write!(pages, "--- Page {}{blank} ---\n{}\n\n", index + 1, page.text);
    }
    prompt::SPLIT_REQUEST.fill(&[("categories", &listed(&categories)), ("pages", pages.trim_end())])
}
