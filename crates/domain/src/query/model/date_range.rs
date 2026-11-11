use chrono::NaiveDate;

use crate::DomainError;
use crate::error::require;

/// Days from [`Self::from`] to [`Self::to`], both included; an open end has no limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DateRange {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

impl DateRange {
    /// # Errors
    /// Fails when the range ends before it starts.
    pub fn new(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<Self, DomainError> {
        let range = Self { from, to };
        range.validate()?;
        Ok(range)
    }

    /// # Errors
    /// Fails when the range ends before it starts.
    pub fn validate(&self) -> Result<(), DomainError> {
        let ordered = match (self.from, self.to) {
            (Some(from), Some(to)) => to >= from,
            _ => true,
        };
        require(ordered, || "The range ends before it starts".to_owned())
    }

    /// The whole of `year`.
    ///
    /// # Errors
    /// Fails for a year outside the supported calendar.
    pub fn year(year: i32) -> Result<Self, DomainError> {
        Ok(Self { from: Some(date(year, 1, 1)?), to: Some(date(year, 12, 31)?) })
    }

    /// `year` and every earlier year.
    ///
    /// # Errors
    /// Fails for a year outside the supported calendar.
    pub fn until_end_of(year: i32) -> Result<Self, DomainError> {
        Ok(Self { from: None, to: Some(date(year, 12, 31)?) })
    }

    #[must_use]
    pub fn contains(&self, date: NaiveDate) -> bool {
        self.from.is_none_or(|from| date >= from) && self.to.is_none_or(|to| date <= to)
    }
}

fn date(year: i32, month: u32, day: u32) -> Result<NaiveDate, DomainError> {
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| DomainError::InvalidArgument(format!("Invalid value for year: {year}")))
}
