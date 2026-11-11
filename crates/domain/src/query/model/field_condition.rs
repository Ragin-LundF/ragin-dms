use std::cmp::Ordering;

use chrono::NaiveDate;
use indexmap::IndexMap;

use crate::DomainError;
use crate::dates;
use crate::error::require;
use crate::field::{Decimal, FieldKey, FieldValue, FieldValues};
use crate::query::FieldComparison;
use crate::text;

/// One condition on a custom field, such as `amount > 100` or `due before 2026-12-01`. [`Self::value`] is needed
/// by every comparison except [`FieldComparison::Present`] and [`FieldComparison::Absent`]. Values of different
/// types never match, except that a number compares with a money amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldCondition {
    pub key: FieldKey,
    pub comparison: FieldComparison,
    pub value: Option<FieldValue>,
}

impl FieldCondition {
    /// # Errors
    /// Fails like [`Self::validate`].
    pub fn new(key: FieldKey, comparison: FieldComparison, value: Option<FieldValue>) -> Result<Self, DomainError> {
        let condition = Self { key, comparison, value };
        condition.validate()?;
        Ok(condition)
    }

    /// # Errors
    /// Fails when a comparison that needs a value has none, or [`FieldComparison::WithinDays`] gets no number.
    pub fn validate(&self) -> Result<(), DomainError> {
        let needs_value = !matches!(self.comparison, FieldComparison::Present | FieldComparison::Absent);
        require(!needs_value || self.value.is_some(), || {
            format!("{} needs a value", comparison_name(self.comparison))
        })?;
        require(
            self.comparison != FieldComparison::WithinDays || matches!(self.value, Some(FieldValue::Number(_))),
            || "WITHIN_DAYS needs a number of days".to_owned(),
        )?;
        self.value.as_ref().map_or(Ok(()), FieldValue::validate)
    }

    #[must_use]
    pub fn matches(&self, fields: &IndexMap<FieldKey, FieldValue>, today: NaiveDate) -> bool {
        let actual = fields.get(&self.key);
        match self.comparison {
            FieldComparison::Present => actual.is_some(),
            FieldComparison::Absent => actual.is_none(),
            _ => actual.is_some_and(|actual| self.compare(actual, today)),
        }
    }

    fn compare(&self, actual: &FieldValue, today: NaiveDate) -> bool {
        let Some(expected) = self.value.as_ref() else {
            return false;
        };
        match self.comparison {
            FieldComparison::Equals => order(actual, expected) == Some(Ordering::Equal),
            FieldComparison::LessThan => order(actual, expected).unwrap_or(Ordering::Equal) == Ordering::Less,
            FieldComparison::GreaterThan => order(actual, expected).unwrap_or(Ordering::Equal) == Ordering::Greater,
            FieldComparison::Contains => text_of(actual).contains(&text_of(expected)),
            FieldComparison::WithinDays => within_days(actual, expected, today),
            FieldComparison::Present | FieldComparison::Absent => true,
        }
    }
}

fn within_days(actual: &FieldValue, days: &FieldValue, today: NaiveDate) -> bool {
    let (FieldValue::Date(date), FieldValue::Number(days)) = (actual, days) else {
        return false;
    };
    *date >= today && *date <= dates::plus_days(today, days.to_i64_truncated())
}

/// Like `compareTo`, or `None` if the two values cannot be compared.
fn order(actual: &FieldValue, expected: &FieldValue) -> Option<Ordering> {
    if let (Some(actual), Some(expected)) = (number_of(actual), number_of(expected)) {
        return Some(actual.compare_value(expected));
    }
    match (actual, expected) {
        (FieldValue::Date(actual), FieldValue::Date(expected)) => Some(actual.cmp(expected)),
        (FieldValue::Flag(actual), FieldValue::Flag(expected)) => Some(actual.cmp(expected)),
        (FieldValue::Text(_), FieldValue::Text(_)) => Some(text::compare(&text_of(actual), &text_of(expected))),
        _ => None,
    }
}

fn number_of(value: &FieldValue) -> Option<&Decimal> {
    match value {
        FieldValue::Number(number) => Some(number),
        FieldValue::Money { amount, .. } => Some(amount),
        _ => None,
    }
}

fn text_of(value: &FieldValue) -> String {
    text::lowercase(&FieldValues::format(value))
}

fn comparison_name(comparison: FieldComparison) -> &'static str {
    match comparison {
        FieldComparison::Present => "PRESENT",
        FieldComparison::Absent => "ABSENT",
        FieldComparison::Equals => "EQUALS",
        FieldComparison::Contains => "CONTAINS",
        FieldComparison::LessThan => "LESS_THAN",
        FieldComparison::GreaterThan => "GREATER_THAN",
        FieldComparison::WithinDays => "WITHIN_DAYS",
    }
}
