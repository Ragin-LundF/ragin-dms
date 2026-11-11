//! How the dashboard writes sizes, times and dates (Kotlin `FileSizes`, `ActivityTexts.time`, `DateTexts`).

use chrono::{DateTime, NaiveDate, TimeZone, Utc};

const STEP: f64 = 1024.0;
const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
const DAYS_PER_WEEK: i64 = 7;
const DAYS_PER_MONTH: i64 = 30;
const DAYS_PER_YEAR: i64 = 365;

/// 512 B, 3.4 KB, 12.0 MB.
#[allow(clippy::cast_precision_loss)] // a size shown with one decimal
pub fn file_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / STEP;
    let mut unit = 0;
    while value >= STEP && unit < UNITS.len() - 1 {
        value /= STEP;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

/// "dd.MM. HH:mm" in `zone`.
pub fn day_and_time<Tz: TimeZone>(instant: DateTime<Utc>, zone: &Tz) -> String
where
    Tz::Offset: std::fmt::Display,
{
    instant.with_timezone(zone).format("%d.%m. %H:%M").to_string()
}

/// The calendar day of `instant` in `zone` as `YYYY-MM-DD`.
pub fn absolute<Tz: TimeZone>(instant: DateTime<Utc>, zone: &Tz) -> String {
    dms_domain::iso_date(instant.with_timezone(zone).date_naive())
}

/// `instant` relative to `now` by calendar days in `zone`: "today", "yesterday", "3 days ago", "in 2 weeks", …
pub fn relative<Tz: TimeZone>(instant: DateTime<Utc>, now: DateTime<Utc>, zone: &Tz) -> String {
    relative_day(instant.with_timezone(zone).date_naive(), now.with_timezone(zone).date_naive())
}

fn relative_day(date: NaiveDate, today: NaiveDate) -> String {
    let days = (date - today).num_days();
    match days {
        0 => "today".to_owned(),
        -1 => "yesterday".to_owned(),
        1 => "tomorrow".to_owned(),
        _ if days < 0 => format!("{} ago", amount_of(-days)),
        _ => format!("in {}", amount_of(days)),
    }
}

fn amount_of(days: i64) -> String {
    if days < DAYS_PER_WEEK * 2 {
        return plural(days, "day");
    }
    if days < DAYS_PER_MONTH * 2 {
        return plural(days / DAYS_PER_WEEK, "week");
    }
    if days < DAYS_PER_YEAR {
        return plural(days / DAYS_PER_MONTH, "month");
    }
    plural(days / DAYS_PER_YEAR, "year")
}

fn plural(count: i64, unit: &str) -> String {
    if count == 1 { format!("1 {unit}") } else { format!("{count} {unit}s") }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(text: &str) -> NaiveDate {
        dms_domain::parse_iso_date(text).expect("valid date")
    }

    #[test]
    fn sizes_keep_bytes_and_round_above_a_kilobyte() {
        assert_eq!(file_size(512), "512 B");
        assert_eq!(file_size(3482), "3.4 KB");
        assert_eq!(file_size(12 * 1024 * 1024), "12.0 MB");
    }

    #[test]
    fn relative_days_switch_to_weeks_months_and_years() {
        let today = day("2026-03-10");
        assert_eq!(relative_day(today, today), "today");
        assert_eq!(relative_day(day("2026-03-09"), today), "yesterday");
        assert_eq!(relative_day(day("2026-02-25"), today), "13 days ago");
        assert_eq!(relative_day(day("2026-03-24"), today), "in 2 weeks");
        assert_eq!(relative_day(day("2026-01-01"), today), "2 months ago");
        assert_eq!(relative_day(day("2024-03-10"), today), "2 years ago");
    }
}
