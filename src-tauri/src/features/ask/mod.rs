//! "Ask your library" (Kotlin `ui.ask`): questions answered from the documents with their sources, and the index
//! behind it.

mod backend;
pub mod commands;
pub mod dto;
pub(crate) mod state;
mod view;
