use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::DomainError;
use crate::text;

/// A tag, normalized once on creation: NFC, trimmed, inner whitespace collapsed.
///
/// Tags are compared case-insensitively but keep the casing they were created with, so `Tax` and `tax` are the
/// same tag and only one of them is ever stored.
#[derive(Debug, Clone)]
pub struct TagName {
    value: String,
    key: String,
}

impl TagName {
    /// In UTF-16 units, as in Kotlin.
    pub const MAX_LENGTH: usize = 60;

    /// The tag for `raw`.
    ///
    /// # Errors
    /// Fails for a blank or too long tag.
    pub fn of(raw: &str) -> Result<Self, DomainError> {
        let cleaned = clean(raw);
        if cleaned.is_empty() {
            return Err(DomainError::InvalidArgument("Tag must not be blank".to_owned()));
        }
        if text::utf16_len(&cleaned) > Self::MAX_LENGTH {
            return Err(DomainError::InvalidArgument(format!("Tag is longer than {} characters", Self::MAX_LENGTH)));
        }
        Ok(Self::from_clean(cleaned))
    }

    /// The tag for text typed by a user or written by the AI, or `None` if it is blank or too long.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        Self::of(raw).ok()
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    fn from_clean(value: String) -> Self {
        let key = text::lowercase(&value);
        Self { value, key }
    }
}

/// NFC, Kotlin `trim()`, then runs of regex `\s` (ASCII whitespace) collapsed to one space.
fn clean(raw: &str) -> String {
    let normalized = text::nfc(raw);
    let mut cleaned = String::with_capacity(normalized.len());
    let mut in_space = false;
    for character in text::trim(&normalized).chars() {
        if text::is_regex_space(character) {
            if !in_space {
                cleaned.push(' ');
            }
            in_space = true;
        } else {
            cleaned.push(character);
            in_space = false;
        }
    }
    cleaned
}

impl PartialEq for TagName {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for TagName {}

impl Hash for TagName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialOrd for TagName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TagName {
    fn cmp(&self, other: &Self) -> Ordering {
        text::compare(&self.key, &other.key)
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.value)
    }
}
