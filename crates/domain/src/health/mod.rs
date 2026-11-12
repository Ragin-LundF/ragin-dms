//! Archive health: what a check of every copy found ([`HealthJudge`]), when the next check is due, which catalog
//! snapshots are kept ([`SnapshotRetention`]) and what restoring one would change ([`SnapshotDiff`]).

mod health_judge;
pub mod model;
mod snapshot_diff;
mod snapshot_retention;

pub use health_judge::HealthJudge;
pub use model::{CopyObservation, CopyState, HealthFinding, HealthSchedule, PartDiff};
pub use snapshot_diff::SnapshotDiff;
pub use snapshot_retention::SnapshotRetention;
