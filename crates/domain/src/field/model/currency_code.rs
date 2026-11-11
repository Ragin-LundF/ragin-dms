use std::fmt;

use crate::DomainError;
use crate::text;

/// An ISO 4217 currency code such as `EUR`. The library's default currency is a setting the user chooses; no
/// currency is assumed anywhere in the logic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// # Errors
    /// Fails for anything but three uppercase letters A-Z.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if is_code(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidArgument(format!("Not an ISO 4217 currency code: {value}")))
        }
    }

    /// The code in `raw`, ignoring case and surrounding blanks, or `None` if it is none.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let code = text::trim(raw).to_uppercase();
        is_code(&code).then_some(Self(code))
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

fn is_code(value: &str) -> bool {
    value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase())
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
