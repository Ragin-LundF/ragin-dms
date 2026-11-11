//! The library's presenters: pure functions from the catalog and the view parameters to DTOs (the part of Kotlin's
//! `LibraryViewModel`, `LibraryStates`, `FolderBrowser` and friends that derived state).

pub mod facets;
pub mod filter;
pub mod folders;
pub mod rows;
pub mod state;
pub mod tokens;
pub mod trees;
