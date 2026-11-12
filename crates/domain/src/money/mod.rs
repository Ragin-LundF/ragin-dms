//! Money: amounts with their currency, sums per currency and the conversion between billing periods. Formatting
//! stays in the UI.

mod billing_period;
pub mod model;

pub use billing_period::BillingPeriod;
pub use model::{Money, MoneyTotals};
