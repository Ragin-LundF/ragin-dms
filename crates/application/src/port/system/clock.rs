use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;

    fn today(&self) -> NaiveDate;

    /// The time zone that turns an instant into a day, as [`Self::today`] does.
    fn zone(&self) -> Tz;
}
