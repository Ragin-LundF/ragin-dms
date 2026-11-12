use indexmap::IndexMap;

use crate::field::{CurrencyCode, Decimal, FieldValue};
use crate::money::Money;

/// Sums of amounts, one per currency, in the order the currencies first appeared. Amounts of different currencies
/// are never added up.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MoneyTotals {
    totals: IndexMap<CurrencyCode, Decimal>,
}

impl MoneyTotals {
    #[must_use]
    pub fn of<'m>(amounts: impl IntoIterator<Item = &'m Money>) -> Self {
        amounts.into_iter().fold(Self::default(), Self::plus)
    }

    /// The sums of the money values among `values`; other values are skipped.
    #[must_use]
    pub fn of_field_values<'v>(values: impl IntoIterator<Item = &'v FieldValue>) -> Self {
        values.into_iter().filter_map(Money::of_field).fold(Self::default(), |totals, money| totals.plus(&money))
    }

    #[must_use]
    pub fn plus(mut self, money: &Money) -> Self {
        let sum = self.totals.get(&money.currency).map_or_else(|| money.amount.clone(), |sum| sum.plus(&money.amount));
        self.totals.insert(money.currency.clone(), sum);
        self
    }

    #[must_use]
    pub fn get(&self, currency: &CurrencyCode) -> Option<&Decimal> {
        self.totals.get(currency)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.totals.is_empty()
    }

    /// The sums as money values, in the order the currencies first appeared.
    #[must_use]
    pub fn amounts(&self) -> Vec<Money> {
        self.totals
            .iter()
            .map(|(currency, amount)| Money { amount: amount.clone(), currency: currency.clone() })
            .collect()
    }
}
