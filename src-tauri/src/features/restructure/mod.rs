//! The restructure assistant (Kotlin `ui.restructure`): scopes, the path template, the plan as move groups, apply,
//! reverse and the removal of folders left empty.

mod backend;
pub mod commands;
pub mod dto;
pub(crate) mod state;
mod view;
