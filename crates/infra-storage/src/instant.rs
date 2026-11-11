//! Instants in the text form of Java's `Instant.toString` / `Instant.parse`, as every file stores them.

use chrono::{DateTime, SecondsFormat, Utc};

use crate::yaml::YamlError;

/// `2024-01-01T10:00:00Z`, with fractions of a second in groups of three digits as needed.
pub fn format(instant: DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

/// # Errors
/// Fails for text that is no ISO-8601 instant.
pub fn parse(text: &str) -> Result<DateTime<Utc>, YamlError> {
    DateTime::parse_from_rfc3339(text)
        .map(|instant| instant.with_timezone(&Utc))
        .map_err(|error| YamlError(format!("Text '{text}' could not be parsed: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instants_round_trip_in_javas_form() {
        for text in ["2024-01-01T10:00:00Z", "2024-01-01T10:00:00.120Z", "2024-01-01T10:00:00.000001Z"] {
            assert_eq!(format(parse(text).expect("valid")), text);
        }
        assert_eq!(format(parse("2024-01-01T12:00:00+02:00").expect("offset")), "2024-01-01T10:00:00Z");
        assert!(parse("2024-01-01").is_err());
    }
}
