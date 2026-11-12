//! `plain_search`: "Turn into filters" in the library's search field. The AI turns a sentence into the library's
//! ordinary facet filters; it never searches by itself. Shown only while "Use AI" is on.

pub mod commands;
pub mod dto;
pub(crate) mod view;
