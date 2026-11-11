use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use dms_application::port::system::Clock;

use crate::support::test_data::instant;

pub struct FixedClock {
    instant: DateTime<Utc>,
}

impl FixedClock {
    pub fn new(instant: DateTime<Utc>) -> Self {
        Self { instant }
    }
}

impl Default for FixedClock {
    fn default() -> Self {
        Self::new(instant("2024-03-01T09:00:00Z"))
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.instant
    }

    fn today(&self) -> NaiveDate {
        self.instant.date_naive()
    }

    fn zone(&self) -> Tz {
        Tz::UTC
    }
}
