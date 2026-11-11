use chrono::NaiveDate;

/// Finds the date a file name carries, the way people commonly write it: `2024-03-15`, `2024_03_15`,
/// `2024.03.15`, `20240315`, `15.03.2024`, `15-03-2024`, or `2024-03` for a month. The leftmost match wins;
/// numbers that are no valid date are ignored.
///
/// The Kotlin version uses three regular expressions with look-arounds and back-references; this scanner
/// reproduces their non-overlapping, left-to-right matches by hand.
pub struct FileNameDates;

const FIRST_YEAR: i32 = 1900;
const LAST_YEAR: i32 = 2199;

/// One regex match: where it starts, how long it is, and the date it names (if valid).
struct Found {
    start: usize,
    length: usize,
    date: Option<NaiveDate>,
}

impl FileNameDates {
    #[must_use]
    pub fn date_in(name: &str) -> Option<NaiveDate> {
        let bytes = name.as_bytes();
        let candidates = [year_first as fn(&[u8], usize) -> Option<Found>, day_first, month_only];
        candidates
            .iter()
            .flat_map(|pattern| find_all(bytes, *pattern))
            .filter_map(|found| found.date.map(|date| (found.start, date)))
            .min_by_key(|(start, _)| *start)
            .map(|(_, date)| date)
    }
}

fn find_all(bytes: &[u8], pattern: fn(&[u8], usize) -> Option<Found>) -> Vec<Found> {
    let mut matches = Vec::new();
    let mut position = 0;
    while position < bytes.len() {
        match pattern(bytes, position) {
            Some(found) => {
                position = found.start + found.length.max(1);
                matches.push(found);
            }
            None => position += 1,
        }
    }
    matches
}

fn digit_at(bytes: &[u8], index: usize) -> bool {
    bytes.get(index).is_some_and(u8::is_ascii_digit)
}

/// `(?<!\d)` at `start`.
fn starts_number(bytes: &[u8], start: usize) -> bool {
    start == 0 || !digit_at(bytes, start - 1)
}

fn digits(bytes: &[u8], start: usize, count: usize) -> Option<&str> {
    let slice = bytes.get(start..start + count)?;
    if slice.iter().all(u8::is_ascii_digit) { std::str::from_utf8(slice).ok() } else { None }
}

/// `(?<!\d)(\d{4})([-_.]?)(\d{2})\2(\d{2})(?!\d)`
fn year_first(bytes: &[u8], start: usize) -> Option<Found> {
    if !starts_number(bytes, start) {
        return None;
    }
    let year = digits(bytes, start, 4)?;
    let separator = bytes.get(start + 4).copied().filter(|byte| matches!(byte, b'-' | b'_' | b'.'));
    // The optional separator is greedy: try it first, then without it.
    [separator.map(|byte| vec![byte]), Some(Vec::new())].into_iter().flatten().find_map(|separator| {
        let month_at = start + 4 + separator.len();
        let month = digits(bytes, month_at, 2)?;
        let day_at = month_at + 2 + separator.len();
        if bytes.get(month_at + 2..day_at)? != separator.as_slice() {
            return None;
        }
        let day = digits(bytes, day_at, 2)?;
        if digit_at(bytes, day_at + 2) {
            return None;
        }
        Some(Found { start, length: day_at + 2 - start, date: date_of(year, month, day) })
    })
}

/// `(?<!\d)(\d{2})([.-])(\d{2})\2(\d{4})(?!\d)`
fn day_first(bytes: &[u8], start: usize) -> Option<Found> {
    if !starts_number(bytes, start) {
        return None;
    }
    let day = digits(bytes, start, 2)?;
    let separator = *bytes.get(start + 2).filter(|byte| matches!(byte, b'.' | b'-'))?;
    let month = digits(bytes, start + 3, 2)?;
    if bytes.get(start + 5) != Some(&separator) {
        return None;
    }
    let year = digits(bytes, start + 6, 4)?;
    if digit_at(bytes, start + 10) {
        return None;
    }
    Some(Found { start, length: 10, date: date_of(year, month, day) })
}

/// `(?<!\d)(\d{4})[-_.](\d{2})(?![-_.]?\d)`
fn month_only(bytes: &[u8], start: usize) -> Option<Found> {
    if !starts_number(bytes, start) {
        return None;
    }
    let year = digits(bytes, start, 4)?;
    if !matches!(bytes.get(start + 4), Some(b'-' | b'_' | b'.')) {
        return None;
    }
    let month = digits(bytes, start + 5, 2)?;
    let after = start + 7;
    let separator_then_digit = matches!(bytes.get(after), Some(b'-' | b'_' | b'.')) && digit_at(bytes, after + 1);
    if digit_at(bytes, after) || separator_then_digit {
        return None;
    }
    Some(Found { start, length: 7, date: date_of(year, month, "01") })
}

fn date_of(year: &str, month: &str, day: &str) -> Option<NaiveDate> {
    let year: i32 = year.parse().ok()?;
    if !(FIRST_YEAR..=LAST_YEAR).contains(&year) {
        return None;
    }
    NaiveDate::from_ymd_opt(year, month.parse().ok()?, day.parse().ok()?)
}
