//! The AI's answer as a [`ContractTermsReading`]: all or nothing, so a value the app cannot read never ends up as a
//! half-filled suggestion.

use dms_application::port::assistant::{ContractTermsReading, ReadPrice};
use dms_domain::contract::{ContractPrice, Notice, NoticeUnit};
use dms_domain::field::{CurrencyCode, Decimal};
use dms_domain::money::BillingPeriod;
use dms_domain::parse_iso_date;

use super::contract_terms_dto::{ContractTermsDto, PriceDto};
use crate::classify::suggestion_dto::outermost_object;

/// The reading in the outermost JSON object of `content`; `None` when any value breaks the schema's rules.
pub(crate) fn reading_of(content: &str) -> Option<ContractTermsReading> {
    let answer: ContractTermsDto = serde_json::from_str(outermost_object(content)?).ok()?;
    let prices = answer.prices.iter().map(price_of).collect::<Option<Vec<ReadPrice>>>()?;
    let notice = match (answer.notice_count, answer.notice_unit.as_deref()) {
        (None, None) => None,
        (Some(count), Some(unit)) if count > 0 => Some(Notice { count, unit: unit_of(unit)? }),
        _ => return None,
    };
    let term_end = match answer.term_end.as_deref() {
        None => None,
        Some(text) => Some(parse_iso_date(text.trim())?),
    };
    if answer.renews_by_months == Some(0) {
        return None;
    }
    Some(ContractTermsReading { prices, notice, term_end, renews_by_months: answer.renews_by_months })
}

fn price_of(price: &PriceDto) -> Option<ReadPrice> {
    let amount = Decimal::parse(price.amount.trim()).filter(|amount| amount.compare_value(&Decimal::zero()).is_gt())?;
    let period = match price.period.as_str() {
        "monthly" => BillingPeriod::Monthly,
        "quarterly" => BillingPeriod::Quarterly,
        "halfYearly" => BillingPeriod::HalfYearly,
        "yearly" => BillingPeriod::Yearly,
        _ => return None,
    };
    Some(ReadPrice {
        valid_from: parse_iso_date(price.valid_from.trim())?,
        price: ContractPrice { amount, currency: CurrencyCode::parse(&price.currency)?, period },
        source: price.source,
    })
}

fn unit_of(unit: &str) -> Option<NoticeUnit> {
    match unit {
        "days" => Some(NoticeUnit::Days),
        "weeks" => Some(NoticeUnit::Weeks),
        "months" => Some(NoticeUnit::Months),
        _ => None,
    }
}
