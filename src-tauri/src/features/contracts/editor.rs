//! The editor's form and the AI review as domain values, and back: typed texts are checked here with the domain's
//! parsers, and a value that does not fit is refused with a sentence that names it.

use dms_application::contract::ContractDraft;
use dms_domain::contract::{
    Contract, ContractId, ContractKind, ContractPrice, ContractTerms, Notice, NoticeUnit, PricePoint,
};
use dms_domain::document::DocumentId;
use dms_domain::field::{CurrencyCode, FieldType, FieldValue, FieldValues};
use dms_domain::money::BillingPeriod;
use dms_domain::{iso_date, parse_iso_date, text};

use super::dto::{
    ContractEditorDto, ContractKindDto, ContractNoticeUnitDto, ContractPeriodDto, ContractPricePointDto,
    ContractTermsInputDto,
};

/// The draft the form describes; the price takes `currency` when none is typed.
///
/// # Errors
/// A sentence naming the first value that does not fit.
pub fn draft_of(form: &ContractEditorDto, currency: Option<&CurrencyCode>) -> Result<ContractDraft, String> {
    let id = optional(&form.id).map(|id| ContractId::new(id).map_err(|error| error.to_string())).transpose()?;
    let price = optional(&form.price).map(|raw| price_of(raw, form.period, currency)).transpose()?;
    Ok(ContractDraft {
        id,
        name: text::trim(&form.name).to_owned(),
        correspondent: optional(&form.correspondent).map(str::to_owned),
        kind: kind_of(form.kind),
        price,
        price_valid_from: optional_date(&form.price_valid_from, "The price's start")?,
        terms: ContractTerms {
            term_end: optional_date(&form.term_end, "The term end")?,
            renews_by_months: count(&form.renews_by_months, "The renewal")?,
            notice: count(&form.notice_count, "The notice period")?
                .map(|count| Notice { count, unit: unit_of(form.notice_unit) }),
            cancellable_any_time: form.cancellable_any_time,
        },
        note: optional(&form.note).map(str::to_owned),
        document_ids: document_ids(&form.document_ids)?,
        added_prices: Vec::new(),
    })
}

/// The form of a new contract for `correspondent` with `document_ids`, as a proposal or the quick look opens it.
pub fn new_form(correspondent: &str, document_ids: Vec<String>) -> ContractEditorDto {
    ContractEditorDto {
        id: String::new(),
        name: correspondent.to_owned(),
        correspondent: correspondent.to_owned(),
        kind: ContractKindDto::Other,
        price: String::new(),
        period: ContractPeriodDto::Monthly,
        price_valid_from: String::new(),
        term_end: String::new(),
        renews_by_months: String::new(),
        notice_count: String::new(),
        notice_unit: ContractNoticeUnitDto::Months,
        cancellable_any_time: false,
        note: String::new(),
        document_ids,
    }
}

/// The form that edits `contract`.
pub fn form_of(contract: &Contract) -> ContractEditorDto {
    let terms = &contract.terms;
    ContractEditorDto {
        id: contract.id.to_string(),
        name: contract.name.clone(),
        correspondent: contract.correspondent.clone().unwrap_or_default(),
        kind: kind_dto(contract.kind),
        price: contract.price.as_ref().map(price_text).unwrap_or_default(),
        period: period_dto(contract.price.as_ref().map_or(BillingPeriod::Monthly, |price| price.period)),
        price_valid_from: String::new(),
        term_end: terms.term_end.map(iso_date).unwrap_or_default(),
        renews_by_months: terms.renews_by_months.map(|months| months.to_string()).unwrap_or_default(),
        notice_count: terms.notice.map(|notice| notice.count.to_string()).unwrap_or_default(),
        notice_unit: unit_dto(terms.notice.map_or(NoticeUnit::Months, |notice| notice.unit)),
        cancellable_any_time: terms.cancellable_any_time,
        note: contract.note.clone().unwrap_or_default(),
        document_ids: Vec::new(),
    }
}

/// The price point to accept.
///
/// # Errors
/// A sentence naming the value that does not fit.
pub fn point_of(dto: &ContractPricePointDto, currency: Option<&CurrencyCode>) -> Result<PricePoint, String> {
    Ok(PricePoint {
        valid_from: optional_date(&dto.valid_from, "The price's start")?.ok_or("The price needs a start date")?,
        price: price_of(&dto.price, dto.period, currency)?,
        document_id: optional(&dto.document_id)
            .map(|id| DocumentId::new(id).map_err(|error| error.to_string()))
            .transpose()?,
    })
}

pub fn point_dto(point: &PricePoint) -> ContractPricePointDto {
    ContractPricePointDto {
        valid_from: iso_date(point.valid_from),
        price: price_text(&point.price),
        period: period_dto(point.price.period),
        document_id: point.document_id.as_ref().map(ToString::to_string).unwrap_or_default(),
    }
}

/// `terms` with the accepted parts of `input`; an empty part keeps what `terms` has.
///
/// # Errors
/// A sentence naming the value that does not fit.
pub fn terms_with(terms: &ContractTerms, input: &ContractTermsInputDto) -> Result<ContractTerms, String> {
    let notice = count(&input.notice_count, "The notice period")?
        .map(|count| Notice { count, unit: unit_of(input.notice_unit) });
    Ok(ContractTerms {
        term_end: optional_date(&input.term_end, "The term end")?.or(terms.term_end),
        renews_by_months: count(&input.renews_by_months, "The renewal")?.or(terms.renews_by_months),
        notice: notice.or(terms.notice),
        cancellable_any_time: terms.cancellable_any_time,
    })
}

/// "1 month", "3 months", "2 weeks", "14 days".
pub fn notice_text(notice: Notice) -> String {
    let unit = match (notice.unit, notice.count == 1) {
        (NoticeUnit::Days, true) => "day",
        (NoticeUnit::Days, false) => "days",
        (NoticeUnit::Weeks, true) => "week",
        (NoticeUnit::Weeks, false) => "weeks",
        (NoticeUnit::Months, true) => "month",
        (NoticeUnit::Months, false) => "months",
    };
    format!("{} {unit}", notice.count)
}

/// "/ month", "/ quarter", "/ half year", "/ year".
pub fn period_text(period: BillingPeriod) -> &'static str {
    match period {
        BillingPeriod::Monthly => "/ month",
        BillingPeriod::Quarterly => "/ quarter",
        BillingPeriod::HalfYearly => "/ half year",
        BillingPeriod::Yearly => "/ year",
    }
}

/// The amount with its currency, `44.99 EUR`, as the price field shows and reads it.
pub fn price_text(price: &ContractPrice) -> String {
    FieldValues::format(&FieldValue::Money { amount: price.amount.clone(), currency: price.currency.clone() })
}

fn price_of(raw: &str, period: ContractPeriodDto, currency: Option<&CurrencyCode>) -> Result<ContractPrice, String> {
    let refused = || format!("{raw} is not a price, such as 44.99 or 44.99 EUR");
    match FieldValues::parse(FieldType::Money, text::trim(raw), currency) {
        Some(FieldValue::Money { amount, currency }) => {
            Ok(ContractPrice { amount, currency, period: period_of(period) })
        }
        Some(FieldValue::Text(_) | FieldValue::Number(_) | FieldValue::Date(_) | FieldValue::Flag(_)) | None => {
            Err(refused())
        }
    }
}

fn optional(raw: &str) -> Option<&str> {
    let trimmed = text::trim(raw);
    (!trimmed.is_empty()).then_some(trimmed)
}

fn optional_date(raw: &str, what: &str) -> Result<Option<chrono::NaiveDate>, String> {
    optional(raw).map(|raw| parse_iso_date(raw).ok_or_else(|| format!("{what} is not a date (YYYY-MM-DD)"))).transpose()
}

fn count(raw: &str, what: &str) -> Result<Option<u16>, String> {
    optional(raw)
        .map(|raw| {
            raw.parse::<u16>()
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| format!("{what} must be a whole number above 0"))
        })
        .transpose()
}

fn document_ids(raw: &[String]) -> Result<Vec<DocumentId>, String> {
    raw.iter().map(|id| DocumentId::new(id).map_err(|error| error.to_string())).collect()
}

pub fn kind_of(kind: ContractKindDto) -> ContractKind {
    match kind {
        ContractKindDto::Utility => ContractKind::Utility,
        ContractKindDto::Telecom => ContractKind::Telecom,
        ContractKindDto::Insurance => ContractKind::Insurance,
        ContractKindDto::Subscription => ContractKind::Subscription,
        ContractKindDto::Membership => ContractKind::Membership,
        ContractKindDto::Loan => ContractKind::Loan,
        ContractKindDto::Other => ContractKind::Other,
    }
}

fn kind_dto(kind: ContractKind) -> ContractKindDto {
    match kind {
        ContractKind::Utility => ContractKindDto::Utility,
        ContractKind::Telecom => ContractKindDto::Telecom,
        ContractKind::Insurance => ContractKindDto::Insurance,
        ContractKind::Subscription => ContractKindDto::Subscription,
        ContractKind::Membership => ContractKindDto::Membership,
        ContractKind::Loan => ContractKindDto::Loan,
        ContractKind::Other => ContractKindDto::Other,
    }
}

fn period_of(period: ContractPeriodDto) -> BillingPeriod {
    match period {
        ContractPeriodDto::Monthly => BillingPeriod::Monthly,
        ContractPeriodDto::Quarterly => BillingPeriod::Quarterly,
        ContractPeriodDto::HalfYearly => BillingPeriod::HalfYearly,
        ContractPeriodDto::Yearly => BillingPeriod::Yearly,
    }
}

fn period_dto(period: BillingPeriod) -> ContractPeriodDto {
    match period {
        BillingPeriod::Monthly => ContractPeriodDto::Monthly,
        BillingPeriod::Quarterly => ContractPeriodDto::Quarterly,
        BillingPeriod::HalfYearly => ContractPeriodDto::HalfYearly,
        BillingPeriod::Yearly => ContractPeriodDto::Yearly,
    }
}

fn unit_of(unit: ContractNoticeUnitDto) -> NoticeUnit {
    match unit {
        ContractNoticeUnitDto::Days => NoticeUnit::Days,
        ContractNoticeUnitDto::Weeks => NoticeUnit::Weeks,
        ContractNoticeUnitDto::Months => NoticeUnit::Months,
    }
}

fn unit_dto(unit: NoticeUnit) -> ContractNoticeUnitDto {
    match unit {
        NoticeUnit::Days => ContractNoticeUnitDto::Days,
        NoticeUnit::Weeks => ContractNoticeUnitDto::Weeks,
        NoticeUnit::Months => ContractNoticeUnitDto::Months,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn euro() -> CurrencyCode {
        CurrencyCode::new("EUR").expect("currency")
    }

    fn filled() -> ContractEditorDto {
        ContractEditorDto {
            price: "44,99".to_owned(),
            term_end: "2026-12-31".to_owned(),
            renews_by_months: "12".to_owned(),
            notice_count: "3".to_owned(),
            note: "  ".to_owned(),
            ..new_form("Net Provider", vec!["doc-1".to_owned()])
        }
    }

    #[test]
    fn a_filled_form_becomes_a_draft_with_the_library_currency_and_back() {
        let draft = draft_of(&filled(), Some(&euro())).expect("valid form");

        assert_eq!(draft.id, None);
        assert_eq!(draft.name, "Net Provider");
        assert_eq!(draft.price.as_ref().map(price_text), Some("44.99 EUR".to_owned()));
        assert_eq!(draft.terms.notice, Some(Notice { count: 3, unit: NoticeUnit::Months }));
        assert_eq!(
            draft.terms.next_cancel_by(chrono::NaiveDate::from_ymd_opt(2026, 11, 11).expect("date")),
            chrono::NaiveDate::from_ymd_opt(2027, 9, 30)
        );
        assert_eq!(draft.note, None);
        let mut contract =
            Contract::new(ContractId::new("c-1").expect("id"), "Net Provider", draft.kind).expect("contract");
        contract.price = draft.price.clone();
        contract.terms = draft.terms.clone();
        let form = form_of(&contract);
        assert_eq!(
            (form.price.as_str(), form.term_end.as_str(), form.notice_count.as_str()),
            ("44.99 EUR", "2026-12-31", "3")
        );
    }

    #[test]
    fn values_that_do_not_fit_are_named() {
        let refused = |form: ContractEditorDto| draft_of(&form, Some(&euro())).expect_err("refused");

        assert_eq!(
            refused(ContractEditorDto { price: "cheap".to_owned(), ..filled() }),
            "cheap is not a price, such as 44.99 or 44.99 EUR"
        );
        assert_eq!(
            refused(ContractEditorDto { term_end: "31.12.2026".to_owned(), ..filled() }),
            "The term end is not a date (YYYY-MM-DD)"
        );
        assert_eq!(
            refused(ContractEditorDto { notice_count: "0".to_owned(), ..filled() }),
            "The notice period must be a whole number above 0"
        );
        assert!(
            draft_of(&ContractEditorDto { price: "44.99".to_owned(), ..filled() }, None).is_err(),
            "no currency known"
        );
    }

    #[test]
    fn accepted_terms_fill_only_what_the_ai_read() {
        let known =
            ContractTerms { term_end: None, renews_by_months: Some(12), notice: None, cancellable_any_time: false };
        let input = ContractTermsInputDto {
            term_end: "2026-12-31".to_owned(),
            renews_by_months: String::new(),
            notice_count: "1".to_owned(),
            notice_unit: ContractNoticeUnitDto::Months,
        };

        let terms = terms_with(&known, &input).expect("valid");

        assert_eq!(terms.renews_by_months, Some(12));
        assert_eq!(terms.notice, Some(Notice { count: 1, unit: NoticeUnit::Months }));
        assert_eq!(terms.term_end, chrono::NaiveDate::from_ymd_opt(2026, 12, 31));
    }
}
