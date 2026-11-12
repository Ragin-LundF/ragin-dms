use chrono::{DateTime, TimeDelta, Utc};

/// How often this device checks the archive on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthSchedule {
    Off,
    #[default]
    Weekly,
    Monthly,
}

impl HealthSchedule {
    /// Whether a check should start at `now` when the last complete one finished at `last_finished`.
    #[must_use]
    pub fn is_due(self, last_finished: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
        let interval = match self {
            Self::Off => return false,
            Self::Weekly => TimeDelta::days(7),
            Self::Monthly => TimeDelta::days(30),
        };
        last_finished.is_none_or(|finished| now - finished >= interval)
    }
}
