//! Cases: a matter with a correspondent (an objection, a claim, a request) with its letters, notes and follow-ups.

mod case_edits;
mod case_planner;
mod case_status;
mod entry_direction;
pub mod model;

pub use case_planner::CasePlanner;
pub use case_status::CaseStatus;
pub use entry_direction::EntryDirection;
pub use model::{Case, CaseEntry, CaseId, CaseNote, FollowUp, FollowUpId, FollowUpSource};
