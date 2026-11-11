//! One module per screen area: its commands, DTOs and presenters (pure functions from domain types to DTOs, the
//! part of the Kotlin view models that derived state). Each module is owned by one worker.

pub mod ask;
pub mod dashboard;
pub mod document;
pub mod inbox;
pub mod library;
pub mod pdf;
pub mod restructure;
pub mod settings;
