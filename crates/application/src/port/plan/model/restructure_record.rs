use chrono::{DateTime, Utc};
use dms_domain::restructure::PlannedMove;

/// A restructure as it was applied: the moves that were done, kept in `.dms-plans/` so the user can reverse it
/// later. [`Self::name`] identifies the file, [`Self::template`] says what the user chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestructureRecord {
    pub name: String,
    pub applied_at: DateTime<Utc>,
    pub template: String,
    pub moves: Vec<PlannedMove>,
}
