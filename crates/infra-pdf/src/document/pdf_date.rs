/// A calendar date, as the document date in PDF metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PdfDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl PdfDate {
    /// The date as a PDF date string at midnight UTC, as the document information stores it.
    pub(crate) fn to_pdf_string(self) -> String {
        format!("D:{:04}{:02}{:02}000000Z", self.year, self.month, self.day)
    }

    /// The date at midnight UTC as XMP writes it.
    pub(crate) fn to_xmp_string(self) -> String {
        format!("{:04}-{:02}-{:02}T00:00:00Z", self.year, self.month, self.day)
    }

    /// The UTC date of a PDF date string (`D:YYYYMMDDHHmmSS+HH'mm'`, every part after the year
    /// optional); `None` if it is no date.
    pub(crate) fn parse_utc(text: &str) -> Option<PdfDate> {
        let text = text.trim();
        let text = text.strip_prefix("D:").unwrap_or(text);
        let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
        if digits.len() < 4 {
            return None;
        }
        let part =
            |from: usize, to: usize, default: i64| digits.get(from..to).and_then(|p| p.parse().ok()).unwrap_or(default);
        let (year, month, day) = (part(0, 4, 0), part(4, 6, 1), part(6, 8, 1));
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return None;
        }
        let local_minutes = part(8, 10, 0) * 60 + part(10, 12, 0);
        let utc_minutes = local_minutes - offset_minutes(&text[digits.len()..]);
        let days = days_from_civil(year, month, day) + utc_minutes.div_euclid(24 * 60);
        Some(civil_from_days(days))
    }
}

/// The offset of a time zone suffix such as `Z`, `+02'00'` or `-0530`, in minutes east of UTC.
fn offset_minutes(zone: &str) -> i64 {
    let sign = match zone.chars().next() {
        Some('+') => 1,
        Some('-') => -1,
        _ => return 0,
    };
    let digits: String = zone[1..].chars().filter(char::is_ascii_digit).collect();
    let hours: i64 = digits.get(0..2).and_then(|p| p.parse().ok()).unwrap_or(0);
    let minutes: i64 = digits.get(2..4).and_then(|p| p.parse().ok()).unwrap_or(0);
    sign * (hours * 60 + minutes)
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // month and day are 1..=31, years far within i32
fn civil_from_days(days: i64) -> PdfDate {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    PdfDate { year: year as i32, month: month as u32, day: day as u32 }
}

#[cfg(test)]
mod tests {
    use super::PdfDate;

    fn date(year: i32, month: u32, day: u32) -> PdfDate {
        PdfDate { year, month, day }
    }

    #[test]
    fn pdf_dates_become_utc_dates() {
        assert_eq!(PdfDate::parse_utc("D:20240315000000Z"), Some(date(2024, 3, 15)));
        assert_eq!(PdfDate::parse_utc("D:20240315000000+00'00'"), Some(date(2024, 3, 15)));
        assert_eq!(PdfDate::parse_utc("D:20240301013000+02'00'"), Some(date(2024, 2, 29)));
        assert_eq!(PdfDate::parse_utc("D:20231231230000-05'00'"), Some(date(2024, 1, 1)));
        assert_eq!(PdfDate::parse_utc("D:2024"), Some(date(2024, 1, 1)));
        assert_eq!(PdfDate::parse_utc("20240315"), Some(date(2024, 3, 15)));
        assert_eq!(PdfDate::parse_utc("yesterday"), None);
        assert_eq!(PdfDate::parse_utc("D:20241315"), None);
    }

    #[test]
    fn a_date_is_written_at_midnight_utc() {
        let written = PdfDate { year: 2024, month: 3, day: 5 };

        assert_eq!(written.to_pdf_string(), "D:20240305000000Z");
        assert_eq!(PdfDate::parse_utc(&written.to_pdf_string()), Some(written));
    }
}
