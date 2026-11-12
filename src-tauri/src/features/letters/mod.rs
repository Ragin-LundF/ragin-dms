//! letters: drafting a letter from documents (by the AI or from a template) and saving it as a catalogued PDF. The
//! window's `LetterDialog` calls these commands; features open it with the documents a letter refers to.

pub mod commands;
pub mod dto;
pub(crate) mod view;
