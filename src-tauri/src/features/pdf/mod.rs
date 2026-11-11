//! The PDF tools of quick look (Kotlin `ui.pdf`): edit pages, split, merge, make searchable, and "Convert to real
//! PDF" with its comparison.

mod backend;
pub mod commands;
pub mod dto;
pub mod rebuild_commands;
mod rebuild_view;
pub(crate) mod state;
mod view;
