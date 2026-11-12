//! settings: the Settings screen — library folders of this device, scanning, inbox, custom fields, matching rules
//! and AI (Kotlin `ui/settings/**`), plus the owner's details for letters.

mod ai;
pub mod commands;
pub(crate) mod dto;
mod folders;
pub(crate) mod library_settings;
pub(crate) mod owner;
pub(crate) mod run;
