use std::fmt;

use crate::DomainError;

/// SHA-256 of a file's content as 64 lowercase hex characters. It is how a moved or renamed file is recognized
/// again, and how two library folders are compared without trusting modification times.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    /// # Errors
    /// Fails for anything but 64 lowercase hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        let valid = value.len() == 64 && value.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
        if valid {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidArgument(format!("Not a lowercase SHA-256 hex digest: {value}")))
        }
    }

    /// # Errors
    /// Fails for anything but 64 hex characters of any case.
    pub fn from_hex(hex: &str) -> Result<Self, DomainError> {
        Self::new(hex.to_lowercase())
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
