use crate::field::Decimal;

/// How often a price is charged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BillingPeriod {
    Monthly,
    Quarterly,
    HalfYearly,
    Yearly,
}

impl BillingPeriod {
    /// The months one period lasts.
    #[must_use]
    pub fn months(self) -> u32 {
        match self {
            Self::Monthly => 1,
            Self::Quarterly => 3,
            Self::HalfYearly => 6,
            Self::Yearly => 12,
        }
    }

    /// What `amount` per period costs per month: exact for monthly prices, otherwise rounded half up to cents.
    #[must_use]
    pub fn monthly(self, amount: &Decimal) -> Decimal {
        match self {
            Self::Monthly => amount.clone(),
            Self::Quarterly | Self::HalfYearly | Self::Yearly => amount.divided_rounded(self.months(), 2),
        }
    }

    /// What `amount` per period costs per year; always exact.
    #[must_use]
    pub fn yearly(self, amount: &Decimal) -> Decimal {
        amount.times(12 / self.months())
    }
}
