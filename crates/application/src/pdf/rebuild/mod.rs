//! "Convert to real PDF": rebuilding scans with the AI and the user's choice for the result.

mod keep_rebuild;
mod line_repair;
pub mod model;
mod rebuild_availability;
mod rebuild_pdf;

pub use keep_rebuild::KeepRebuild;
pub use line_repair::LineRepair;
pub use model::{RebuildBlocker, RebuildChoice, RebuildOutcome};
pub use rebuild_availability::RebuildAvailability;
pub use rebuild_pdf::RebuildPdf;
