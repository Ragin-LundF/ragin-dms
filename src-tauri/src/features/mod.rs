//! One module per screen area: its commands, DTOs and presenters (pure functions from domain types to DTOs, the
//! part of the Kotlin view models that derived state). Each module is owned by one worker.

pub mod ask;
pub mod bills;
pub mod cases;
pub mod contracts;
pub mod dashboard;
pub mod document;
pub mod emergency;
pub mod health;
pub mod inbox;
pub mod letters;
pub mod library;
pub mod organize;
pub mod pdf;
pub mod plain_search;
pub mod restructure;
pub mod settings;
pub mod split;
pub mod tax;
