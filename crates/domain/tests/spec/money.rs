use dms_domain::contract::ContractPrice;
use dms_domain::field::{CurrencyCode, FieldValue};
use dms_domain::money::{BillingPeriod, Money, MoneyTotals};

use crate::test_data::{date, decimal, euro, euros};

fn dollars(amount: &str) -> Money {
    Money { amount: decimal(amount), currency: CurrencyCode::new("USD").expect("valid currency") }
}

#[test]
fn totals_add_up_each_currency_on_its_own_in_the_order_seen() {
    let totals = MoneyTotals::of(&[euros("12.30"), dollars("5"), euros("0.70"), euros("100")]);

    assert_eq!(totals.amounts(), vec![euros("113.00"), dollars("5")]);
    assert_eq!(totals.get(&euro()), Some(&decimal("113.00")));
}

#[test]
fn totals_of_field_values_skip_everything_but_money() {
    let values = [
        FieldValue::Money { amount: decimal("19.99"), currency: euro() },
        FieldValue::Number(decimal("7")),
        FieldValue::Date(date(2026, 1, 1)),
        FieldValue::Money { amount: decimal("0.01"), currency: euro() },
    ];

    assert_eq!(MoneyTotals::of_field_values(&values).amounts(), vec![euros("20.00")]);
    assert!(MoneyTotals::of_field_values(&[FieldValue::Flag(true)]).is_empty());
}

#[test]
fn periods_convert_to_month_and_year_exactly_or_rounded_half_up() {
    assert_eq!(BillingPeriod::Monthly.monthly(&decimal("29.99")), decimal("29.99"));
    assert_eq!(BillingPeriod::Monthly.yearly(&decimal("29.99")), decimal("359.88"));
    assert_eq!(BillingPeriod::Quarterly.monthly(&decimal("100")), decimal("33.33"));
    assert_eq!(BillingPeriod::Quarterly.yearly(&decimal("100")), decimal("400"));
    assert_eq!(BillingPeriod::HalfYearly.monthly(&decimal("100.03")), decimal("16.67"));
    assert_eq!(BillingPeriod::Yearly.monthly(&decimal("487.20")), decimal("40.60"));
    assert_eq!(BillingPeriod::Yearly.yearly(&decimal("487.20")), decimal("487.20"));
}

#[test]
fn a_contract_price_keeps_its_currency_when_converted() {
    let price = ContractPrice { amount: decimal("120"), currency: euro(), period: BillingPeriod::Yearly };

    assert_eq!(price.monthly(), euros("10.00"));
    assert_eq!(price.yearly(), euros("120"));
}
