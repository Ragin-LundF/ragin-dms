use crate::field::{CurrencyCode, Decimal, FieldValue};

/// An exact amount in one currency, like a [`FieldValue::Money`] value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    pub amount: Decimal,
    pub currency: CurrencyCode,
}

impl Money {
    /// The amount of a money field value; `None` for any other value.
    #[must_use]
    pub fn of_field(value: &FieldValue) -> Option<Self> {
        match value {
            FieldValue::Money { amount, currency } => Some(Self { amount: amount.clone(), currency: currency.clone() }),
            FieldValue::Text(_) | FieldValue::Number(_) | FieldValue::Date(_) | FieldValue::Flag(_) => None,
        }
    }
}
