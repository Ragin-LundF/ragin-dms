use std::fmt;

use crate::DomainError;
use crate::text;

/// Identifies a custom field across the library, for example `amount` or `due`. Lowercase letters, digits, `-`
/// and `_`, starting with a letter, so it reads well in the catalog file and in path templates (`{field:due}`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldKey(String);

impl FieldKey {
    pub const MAX_LENGTH: usize = 40;

    /// # Errors
    /// Fails for anything but `[a-z][a-z0-9_-]{0,39}`.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if is_key(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidArgument(format!("Not a valid field key: {value}")))
        }
    }

    /// A key derived from a label such as `Contract end`, or `None` if nothing usable is left.
    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        let lowered = text::lowercase(text::trim(label));
        let mut key = String::with_capacity(lowered.len());
        let mut in_separator = false;
        for character in lowered.chars() {
            if text::is_regex_space(character) || character == '_' {
                if !in_separator {
                    key.push('-');
                }
                in_separator = true;
            } else {
                in_separator = false;
                if matches!(character, 'a'..='z' | '0'..='9' | '_' | '-') {
                    key.push(character);
                }
            }
        }
        let key = key.trim_start_matches(['-', '_']).trim_end_matches('-');
        let key = text::take_utf16(key, Self::MAX_LENGTH);
        is_key(key).then(|| Self(key.to_owned()))
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

fn is_key(value: &str) -> bool {
    let mut bytes = value.bytes();
    let starts_with_letter = bytes.next().is_some_and(|byte| byte.is_ascii_lowercase());
    starts_with_letter
        && value.len() <= FieldKey::MAX_LENGTH
        && bytes.all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'))
}

impl fmt::Display for FieldKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
