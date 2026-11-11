//! The restructure assistant: planning, applying and reversing restructures.

mod apply_restructure_plan;
mod list_restructure_history;
pub mod model;
mod plan_restructure;
mod remove_empty_folders;
mod reverse_restructure;

pub use apply_restructure_plan::ApplyRestructurePlan;
pub use list_restructure_history::ListRestructureHistory;
pub use model::{PlanOutcome, RestructureHistoryOutcome, RestructureOutcome, RestructureScope};
pub use plan_restructure::PlanRestructure;
pub use remove_empty_folders::RemoveEmptyFolders;
pub use reverse_restructure::ReverseRestructure;
