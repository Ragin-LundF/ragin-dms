//! Date arithmetic like `java.time.LocalDate`, which throws only far beyond any real date. chrono's range is
//! smaller, so results outside it are clamped to the first or last date chrono knows.

use chrono::{Datelike, Days, NaiveDate};

#[must_use]
pub fn plus_days(date: NaiveDate, days: i64) -> NaiveDate {
    let magnitude = Days::new(days.unsigned_abs());
    let moved = if days >= 0 { date.checked_add_days(magnitude) } else { date.checked_sub_days(magnitude) };
    moved.unwrap_or(if days >= 0 { NaiveDate::MAX } else { NaiveDate::MIN })
}

#[must_use]
pub fn minus_days(date: NaiveDate, days: i64) -> NaiveDate {
    plus_days(date, days.saturating_neg())
}

/// December 31 of `year`, clamped like [`plus_days`].
#[must_use]
pub fn last_day_of(year: i64) -> NaiveDate {
    i32::try_from(year).ok().and_then(|year| NaiveDate::from_ymd_opt(year, 12, 31)).unwrap_or(if year > 0 {
        NaiveDate::MAX
    } else {
        NaiveDate::MIN
    })
}

/// `LocalDate.toString()`: `2024-03-05`, with a sign and more digits outside 0000..9999.
#[must_use]
pub fn iso(date: NaiveDate) -> String {
    let year = date.year();
    let month = date.month();
    let day = date.day();
    if (0..=9999).contains(&year) {
        format!("{year:04}-{month:02}-{day:02}")
    } else if year < 0 {
        format!("-{:04}-{month:02}-{day:02}", year.unsigned_abs())
    } else {
        format!("+{year}-{month:02}-{day:02}")
    }
}

/// `LocalDate.parse` with the ISO format: `yyyy-MM-dd`; a year beyond four digits needs a sign.
#[must_use]
pub fn parse_iso(text: &str) -> Option<NaiveDate> {
    let (sign, rest) = match text.as_bytes().first()? {
        b'+' => (Some('+'), &text[1..]),
        b'-' => (Some('-'), &text[1..]),
        _ => (None, text),
    };
    let mut parts = rest.split('-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    let year_ok = digits(year)
        && (4..=10).contains(&year.len())
        && match sign {
            None => year.len() == 4,
            Some('+') => year.len() > 4,
            _ => true,
        };
    if parts.next().is_some() || !year_ok || month.len() != 2 || day.len() != 2 || !digits(month) || !digits(day) {
        return None;
    }
    let year: i32 = year.parse().ok()?;
    let year = if sign == Some('-') { -year } else { year };
    NaiveDate::from_ymd_opt(year, month.parse().ok()?, day.parse().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_round_trip_follows_java() {
        let date = NaiveDate::from_ymd_opt(2026, 12, 1).expect("valid");
        assert_eq!(iso(date), "2026-12-01");
        assert_eq!(parse_iso("2026-12-01"), Some(date));
        for bad in ["2026-2-01", "26-12-01", "+2026-12-01", "2026-02-30", "2026-12-01x", ""] {
            assert_eq!(parse_iso(bad), None, "{bad}");
        }
        let far = parse_iso("+10000-01-01").expect("signed long year");
        assert_eq!(iso(far), "+10000-01-01");
    }

    #[test]
    fn arithmetic_clamps_instead_of_failing() {
        assert_eq!(plus_days(NaiveDate::MAX, 1), NaiveDate::MAX);
        assert_eq!(minus_days(NaiveDate::MIN, 1), NaiveDate::MIN);
    }
}
