use chrono::{DateTime, Utc};

use crate::port::state::ActivityKind;

/// One line of the activity log. [`Self::subject`] names what was affected, usually a path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActivityEntry {
    pub at: DateTime<Utc>,
    pub kind: ActivityKind,
    pub subject: String,
}
