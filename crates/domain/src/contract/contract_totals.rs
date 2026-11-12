use chrono::NaiveDate;

use crate::contract::Contract;
use crate::dates;
use crate::money::{Money, MoneyTotals};

/// What the running contracts with a price cost together, per currency: per month, per year, the change of the
/// monthly sum against the same day a year ago by the price histories, and how many prices rose within that year.
/// A contract without a price point a year ago counts with its current price then, so it changes nothing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContractTotals {
    pub monthly: MoneyTotals,
    pub yearly: MoneyTotals,
    pub monthly_change: MoneyTotals,
    pub price_increases: usize,
}

impl ContractTotals {
    #[must_use]
    pub fn of<'c>(contracts: impl IntoIterator<Item = &'c Contract>, today: NaiveDate) -> Self {
        let year_ago = dates::minus_days(today, 365);
        contracts.into_iter().filter(|contract| contract.is_running(today)).fold(Self::default(), |totals, contract| {
            let Some(price) = &contract.price else { return totals };
            let monthly = price.monthly();
            let before = contract.price_on(year_ago).filter(|old| old.currency == price.currency).unwrap_or(price);
            let change = Money {
                amount: monthly.amount.plus(&before.monthly().amount.negated()),
                currency: monthly.currency.clone(),
            };
            Self {
                monthly: totals.monthly.plus(&monthly),
                yearly: totals.yearly.plus(&price.yearly()),
                monthly_change: totals.monthly_change.plus(&change),
                price_increases: totals.price_increases + usize::from(contract.recent_increase(today).is_some()),
            }
        })
    }
}
