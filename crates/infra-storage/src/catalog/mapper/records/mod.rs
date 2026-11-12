//! The parts added in 2.1: contracts, cases, tax years and tax assignments, the owner, expectations and the
//! emergency folder. Records are sorted by id (tax years by year) so the file is stable; the entries inside a record
//! keep their order.

pub mod case_mapper;
pub mod contract_mapper;
pub mod organize_mapper;
pub mod tax_mapper;

use dms_domain::field::CurrencyCode;
use dms_domain::money::Money;

use crate::catalog::dto::tax::MoneyDto;
use crate::catalog::mapper::field_mapper;
use crate::yaml::YamlError;

fn money_to_dto(money: &Money) -> MoneyDto {
    MoneyDto { amount: money.amount.to_plain_string(), currency: money.currency.value().to_owned() }
}

fn money_to_domain(dto: &MoneyDto) -> Result<Money, YamlError> {
    Ok(Money { amount: field_mapper::decimal(&dto.amount)?, currency: CurrencyCode::new(dto.currency.as_str())? })
}

/// A whole number of the file in the range of `T`.
fn number<T: TryFrom<i64>>(value: i64, what: &str) -> Result<T, YamlError> {
    T::try_from(value).map_err(|_| YamlError(format!("{what} out of range: {value}")))
}
