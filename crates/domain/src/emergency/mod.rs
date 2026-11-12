//! The emergency folder: what a partner or relative needs if something happens to the user.

mod emergency_changes;
mod emergency_group;
mod emergency_planner;
pub mod model;

pub use emergency_changes::EmergencyChanges;
pub use emergency_group::EmergencyGroup;
pub use emergency_planner::EmergencyPlanner;
pub use model::{EmergencyEntry, EmergencyFolder, EmergencyItem, EmergencyItemId};
