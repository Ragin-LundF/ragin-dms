use chrono::NaiveDate;

use crate::DomainError;
use crate::error::require;
use crate::field::{CurrencyCode, Decimal, FieldType};
use crate::text;

/// The value of a custom field on one document. Its [`Self::field_type`] must match the field's definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldValue {
    /// Never blank; see [`Self::text`].
    Text(String),
    Number(Decimal),
    Money {
        amount: Decimal,
        currency: CurrencyCode,
    },
    Date(NaiveDate),
    Flag(bool),
}

impl FieldValue {
    /// A text value.
    ///
    /// # Errors
    /// Fails for blank text.
    pub fn text(text: impl Into<String>) -> Result<Self, DomainError> {
        let value = Self::Text(text.into());
        value.validate()?;
        Ok(value)
    }

    /// # Errors
    /// Fails for blank text.
    pub fn validate(&self) -> Result<(), DomainError> {
        match self {
            Self::Text(text) => require(!text::is_blank(text), || "A text value must not be blank".to_owned()),
            _ => Ok(()),
        }
    }

    #[must_use]
    pub fn field_type(&self) -> FieldType {
        match self {
            Self::Text(_) => FieldType::Text,
            Self::Number(_) => FieldType::Number,
            Self::Money { .. } => FieldType::Money,
            Self::Date(_) => FieldType::Date,
            Self::Flag(_) => FieldType::Boolean,
        }
    }
}
