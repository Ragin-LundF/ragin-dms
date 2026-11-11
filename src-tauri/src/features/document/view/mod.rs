//! Presenters of the document area: pure functions from domain types to DTOs and checked edits, tested on
//! synthetic catalogs.

pub mod details;
pub mod edits;
pub mod quick_look;
pub mod review;
pub mod texts;

#[cfg(test)]
mod test_catalog;
