//! Tauri commands of the document area: parse the input, call a presenter or one use case, map to a DTO.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

pub mod assistant;
pub mod details;
pub mod quick_look;
pub mod review;
pub(crate) mod wiring;
