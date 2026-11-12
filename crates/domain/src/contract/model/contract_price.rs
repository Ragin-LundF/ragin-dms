use crate::field::{CurrencyCode, Decimal};
use crate::money::{BillingPeriod, Money};

/// What a contract costs per period.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContractPrice {
    pub amount: Decimal,
    pub currency: CurrencyCode,
    pub period: BillingPeriod,
}

impl ContractPrice {
    #[must_use]
    pub fn monthly(&self) -> Money {
        Money { amount: self.period.monthly(&self.amount), currency: self.currency.clone() }
    }

    #[must_use]
    pub fn yearly(&self) -> Money {
        Money { amount: self.period.yearly(&self.amount), currency: self.currency.clone() }
    }

    /// `true` when this price costs more per month than `other` in the same currency.
    #[must_use]
    pub fn costs_more_than(&self, other: &Self) -> bool {
        self.currency == other.currency && self.monthly().amount.compare_value(&other.monthly().amount).is_gt()
    }
}
