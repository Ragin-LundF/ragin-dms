//! Amounts and areas as the tax screens show them; the UI formats the amounts.

use chrono::{Datelike, NaiveDate};
use dms_domain::field::{FieldValue, FieldValues};
use dms_domain::money::Money;
use dms_domain::tax::TaxArea;
use dms_domain::text;

use crate::features::tax::dto::{TaxAreaDto, TaxAreaOptionDto};

/// An amount as the review and the edit dialog show it, such as `684.50 EUR`.
pub fn money_text(money: &Money) -> String {
    FieldValues::format(&FieldValue::Money { amount: money.amount.clone(), currency: money.currency.clone() })
}

/// The area whose label is `typed`, ignoring case and surrounding spaces.
pub fn area_of_label(typed: &str) -> Option<TaxArea> {
    let wanted = text::lowercase(text::trim(typed));
    TaxArea::ALL.into_iter().find(|area| text::lowercase(area.label()) == wanted)
}

pub fn area_labels() -> Vec<String> {
    TaxArea::ALL.iter().map(|area| area.label().to_owned()).collect()
}

/// An amount as the tax screens show it: `€ 1,312.40`, with the currency's sign where it has a common one.
pub fn money_display(money: &Money) -> String {
    let plain = money.amount.to_plain_string();
    let (sign, digits) = plain.strip_prefix('-').map_or(("", plain.as_str()), |rest| ("-", rest));
    let (whole, fraction) = digits.split_once('.').map_or((digits, None), |(whole, fraction)| (whole, Some(fraction)));
    let mut grouped = String::new();
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    let number = fraction.map_or(grouped.clone(), |fraction| format!("{grouped}.{fraction}"));
    let symbol = match money.currency.value() {
        "EUR" => "€",
        "USD" => "$",
        "GBP" => "£",
        "CHF" => "CHF",
        other => other,
    };
    format!("{sign}{symbol} {number}")
}

pub fn area_dto(area: TaxArea) -> TaxAreaDto {
    match area {
        TaxArea::Income => TaxAreaDto::Income,
        TaxArea::PensionInsurance => TaxAreaDto::PensionInsurance,
        TaxArea::HouseholdServices => TaxAreaDto::HouseholdServices,
        TaxArea::Donations => TaxAreaDto::Donations,
        TaxArea::HealthCosts => TaxAreaDto::HealthCosts,
        TaxArea::WorkExpenses => TaxAreaDto::WorkExpenses,
        TaxArea::CapitalIncome => TaxAreaDto::CapitalIncome,
        TaxArea::Rental => TaxAreaDto::Rental,
        TaxArea::Other => TaxAreaDto::Other,
    }
}

pub fn area_of(area: TaxAreaDto) -> TaxArea {
    match area {
        TaxAreaDto::Income => TaxArea::Income,
        TaxAreaDto::PensionInsurance => TaxArea::PensionInsurance,
        TaxAreaDto::HouseholdServices => TaxArea::HouseholdServices,
        TaxAreaDto::Donations => TaxArea::Donations,
        TaxAreaDto::HealthCosts => TaxArea::HealthCosts,
        TaxAreaDto::WorkExpenses => TaxArea::WorkExpenses,
        TaxAreaDto::CapitalIncome => TaxArea::CapitalIncome,
        TaxAreaDto::Rental => TaxArea::Rental,
        TaxAreaDto::Other => TaxArea::Other,
    }
}

pub fn area_option(area: TaxArea) -> TaxAreaOptionDto {
    TaxAreaOptionDto { area: area_dto(area), label: area.label().to_owned(), form: area.form().to_owned() }
}

pub fn area_options() -> Vec<TaxAreaOptionDto> {
    TaxArea::ALL.into_iter().map(area_option).collect()
}

/// The year the tax screen opens with: the previous calendar year until 31 July, then the current one.
pub fn default_year(today: NaiveDate) -> i32 {
    if today.month() <= 7 { today.year() - 1 } else { today.year() }
}

#[cfg(test)]
mod tests {
    use dms_domain::field::{CurrencyCode, Decimal};

    use super::*;

    fn money(amount: &str, currency: &str) -> Money {
        Money { amount: Decimal::parse(amount).expect("amount"), currency: CurrencyCode::new(currency).expect("code") }
    }

    #[test]
    fn amounts_get_their_sign_and_thousands_separators() {
        assert_eq!(money_display(&money("684.50", "EUR")), "€ 684.50");
        assert_eq!(money_display(&money("1312.4", "EUR")), "€ 1,312.4");
        assert_eq!(money_display(&money("-1234567", "SEK")), "-SEK 1,234,567");
    }

    #[test]
    fn the_default_year_is_last_year_until_the_end_of_july() {
        let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).expect("date");
        assert_eq!(default_year(day(7, 31)), 2025);
        assert_eq!(default_year(day(8, 1)), 2026);
    }
}
