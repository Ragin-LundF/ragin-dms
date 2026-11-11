//! Resolving detected changes and duplicates: keeping versions, applying or undoing moves, merging copies, and
//! dismissing tasks.

mod dismiss_task;
mod duplicate_preference;
mod keep_both_versions;
mod keep_version;
mod merge_duplicates;
pub mod model;
mod resolve_move;

pub use dismiss_task::DismissTask;
pub use duplicate_preference::DuplicatePreference;
pub use keep_both_versions::KeepBothVersions;
pub use keep_version::KeepVersion;
pub use merge_duplicates::MergeDuplicates;
pub use model::MoveDecision;
pub use resolve_move::ResolveMove;
