//! Expected documents: senders that send documents in a regular rhythm, and the ones that are late.

mod expectation_mode;
mod expectation_planner;
pub mod model;
mod rhythm;
mod rhythm_detector;

pub use expectation_mode::ExpectationMode;
pub use expectation_planner::ExpectationPlanner;
pub use model::{ExpectationOverride, ExpectationSnoozes, ExpectedDocument};
pub use rhythm::Rhythm;
pub use rhythm_detector::RhythmDetector;
