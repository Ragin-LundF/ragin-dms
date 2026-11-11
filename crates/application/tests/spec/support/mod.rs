//! Fakes, the in-memory `TestLibrary` and synthetic test data shared by every feature's tests.
#![allow(dead_code, unused_imports)] // shared fixtures: not every feature uses every helper

pub mod fake;
mod shared;
pub mod test_data;
mod test_library;
mod wiring;

pub use shared::Shared;
pub use test_library::TestLibrary;
