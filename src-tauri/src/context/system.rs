//! The system clock and random ids (Kotlin `SystemClock`, `UUID.randomUUID()`).

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use dms_application::port::system::{Clock, IdGenerator};

pub struct SystemClock {
    zone: Tz,
}

impl SystemClock {
    /// The clock in the system's time zone; UTC if the system names none chrono-tz knows.
    pub fn new() -> Self {
        let zone = iana_time_zone::get_timezone().ok().and_then(|name| name.parse().ok()).unwrap_or(Tz::UTC);
        Self { zone }
    }
}

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn today(&self) -> NaiveDate {
        Utc::now().with_timezone(&self.zone).date_naive()
    }

    fn zone(&self) -> Tz {
        self.zone
    }
}

pub struct RandomIds;

impl IdGenerator for RandomIds {
    fn new_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
