//! Cases: matters with a correspondent, their documents in and out, notes and follow-ups. Every use case changes
//! only the catalog and documents always stay in the library.

mod case_commit;
mod case_services;
mod documents;
mod follow_ups;
mod lifecycle;
pub mod model;

pub use case_services::CaseServices;
pub use documents::{AddToCase, RemoveFromCase};
pub use follow_ups::{AddFollowUp, CompleteFollowUp, SnoozeFollowUp};
pub use lifecycle::{AddCaseNote, CreateCase, DeleteCase, SetCaseStatus};
pub use model::{CreateCaseOutcome, NewCase, NewFollowUp};
