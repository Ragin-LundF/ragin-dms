//! The presenters of the contracts screen: pure functions from domain types to DTOs.

use chrono::{Datelike, NaiveDate};
use dms_application::contract::ContractTermsSuggestion;
use dms_domain::catalog::Catalog;
use dms_domain::contract::{Contract, ContractPlanner, ContractPrice, ContractTotals};
use dms_domain::document::DocumentId;
use dms_domain::field::{Decimal, FieldValue, FieldValues};
use dms_domain::money::Money;
use dms_domain::{iso_date, text};

use super::dto::{
    ContractCardDto, ContractChoiceDto, ContractDetailDto, ContractHistoryPointDto, ContractPillDto, ContractPillTone,
    ContractSuggestedPriceDto, ContractTermsInputDto, ContractTermsReviewDto, ContractTotalDto, ContractsOfDocumentDto,
    ContractsStatusDto, ContractsViewDto,
};
use super::editor::{form_of, notice_text, period_text, point_dto, price_text};
use crate::features::contracts::dto::ContractNoticeUnitDto;

/// The chart colours of the logo letters, used in this order and never cycled; then the neutral one.
const CHART_COLOURS: u32 = 4;

pub fn contracts_view(catalog: &Catalog, today: NaiveDate) -> ContractsViewDto {
    let totals = ContractTotals::of(catalog.contracts().values(), today);
    ContractsViewDto {
        totals: totals_of(&totals),
        cards: sorted(catalog, today)
            .into_iter()
            .enumerate()
            .map(|(index, contract)| card_of(contract, index, catalog, today))
            .collect(),
    }
}

pub fn contracts_status(catalog: &Catalog, today: NaiveDate) -> Option<ContractsStatusDto> {
    let totals = ContractTotals::of(catalog.contracts().values(), today);
    if totals.monthly.is_empty() {
        return None;
    }
    let increases = u32::try_from(totals.price_increases).unwrap_or(u32::MAX);
    Some(ContractsStatusDto {
        monthly: totals.monthly.amounts().iter().map(money_text).collect(),
        increases,
        increases_label: match increases {
            0 => None,
            1 => Some("1 price increase".to_owned()),
            count => Some(format!("{count} price increases")),
        },
    })
}

pub fn contracts_of_document(catalog: &Catalog, document_id: &DocumentId, today: NaiveDate) -> ContractsOfDocumentDto {
    let choice = |contract: &Contract| ContractChoiceDto { id: contract.id.to_string(), name: contract.name.clone() };
    let (with, without): (Vec<&Contract>, Vec<&Contract>) =
        sorted(catalog, today).into_iter().partition(|contract| contract.document_ids.contains(document_id));
    ContractsOfDocumentDto {
        contracts: with.into_iter().map(choice).collect(),
        others: without.into_iter().filter(|contract| contract.is_running(today)).map(choice).collect(),
        correspondent: catalog
            .documents()
            .get(document_id)
            .and_then(|document| document.metadata.correspondent.clone()),
    }
}

/// The AI's reading for the review: each price marked where it costs more than the one before it.
pub fn terms_review(
    suggestion: &ContractTermsSuggestion,
    contract: &Contract,
    catalog: &Catalog,
) -> ContractTermsReviewDto {
    let mut known: Vec<&ContractPrice> = Vec::new();
    let prices = suggestion
        .prices
        .iter()
        .map(|point| {
            let before = contract
                .price_history
                .iter()
                .filter(|known| known.valid_from < point.valid_from)
                .max_by_key(|known| known.valid_from)
                .map(|known| &known.price)
                .or_else(|| known.last().copied());
            known.push(&point.price);
            ContractSuggestedPriceDto {
                label: format!(
                    "{} {} from {}",
                    price_text(&point.price),
                    period_text(point.price.period),
                    iso_date(point.valid_from)
                ),
                source: point.document_id.as_ref().and_then(|id| catalog.documents().get(id)).map(|document| {
                    document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned())
                }),
                rose: before.is_some_and(|before| point.price.costs_more_than(before)),
                point: point_dto(point),
            }
        })
        .collect();
    let mut parts = Vec::new();
    if let Some(notice) = suggestion.notice {
        parts.push(format!("Notice {}", notice_text(notice)));
    }
    if let Some(term_end) = suggestion.term_end {
        parts.push(format!("ends {}", iso_date(term_end)));
    }
    if let Some(months) = suggestion.renews_by_months {
        parts.push(format!("renews by {months} months"));
    }
    let terms = (!parts.is_empty()).then(|| ContractTermsInputDto {
        term_end: suggestion.term_end.map(iso_date).unwrap_or_default(),
        renews_by_months: suggestion.renews_by_months.map(|months| months.to_string()).unwrap_or_default(),
        notice_count: suggestion.notice.map(|notice| notice.count.to_string()).unwrap_or_default(),
        notice_unit: match suggestion.notice.map(|notice| notice.unit) {
            Some(dms_domain::contract::NoticeUnit::Days) => ContractNoticeUnitDto::Days,
            Some(dms_domain::contract::NoticeUnit::Weeks) => ContractNoticeUnitDto::Weeks,
            Some(dms_domain::contract::NoticeUnit::Months) | None => ContractNoticeUnitDto::Months,
        },
    });
    ContractTermsReviewDto {
        prices,
        terms_label: (!parts.is_empty()).then(|| parts.join(" · ")),
        terms,
        origin: format!("Read by AI ({})", suggestion.model),
    }
}

/// Running contracts first, each part by name.
fn sorted(catalog: &Catalog, today: NaiveDate) -> Vec<&Contract> {
    let mut contracts: Vec<&Contract> = catalog.contracts().values().collect();
    contracts.sort_by(|first, second| {
        second
            .is_running(today)
            .cmp(&first.is_running(today))
            .then_with(|| text::compare(&text::lowercase(&first.name), &text::lowercase(&second.name)))
    });
    contracts
}

fn totals_of(totals: &ContractTotals) -> Vec<ContractTotalDto> {
    totals
        .monthly
        .amounts()
        .iter()
        .map(|monthly| {
            let change = totals.monthly_change.get(&monthly.currency).cloned().unwrap_or_else(Decimal::zero);
            let sign = change.compare_value(&Decimal::zero());
            ContractTotalDto {
                monthly: money_text(monthly),
                yearly: totals.yearly.get(&monthly.currency).map_or_else(String::new, |yearly| {
                    money_text(&Money { amount: yearly.clone(), currency: monthly.currency.clone() })
                }),
                change: (!sign.is_eq()).then(|| {
                    let magnitude = if sign.is_lt() { change.negated() } else { change };
                    let text = money_text(&Money { amount: magnitude, currency: monthly.currency.clone() });
                    format!("{} {text} / month vs. last year", if sign.is_lt() { "−" } else { "+" })
                }),
                rising: sign.is_gt(),
            }
        })
        .collect()
}

fn card_of(contract: &Contract, index: usize, catalog: &Catalog, today: NaiveDate) -> ContractCardDto {
    let letter = contract.correspondent.as_deref().unwrap_or(&contract.name).trim().chars().next();
    ContractCardDto {
        id: contract.id.to_string(),
        name: contract.name.clone(),
        correspondent: contract.correspondent.clone(),
        letter: letter.map(|letter| letter.to_uppercase().collect()).unwrap_or_default(),
        color: u32::try_from(index).unwrap_or(u32::MAX).min(CHART_COLOURS),
        monthly: contract.price.as_ref().map(|price| money_text(&price.monthly())),
        pills: pills_of(contract, today),
        running: contract.is_running(today),
        detail: detail_of(contract, catalog, today),
    }
}

fn pills_of(contract: &Contract, today: NaiveDate) -> Vec<ContractPillDto> {
    let pill = |label: String, tone| ContractPillDto { label, tone };
    let mut pills = Vec::new();
    if contract.recent_increase(today).is_some() {
        pills.push(pill("Price ↑".to_owned(), ContractPillTone::Warning));
    }
    if let Some(ended_on) = contract.ended_on {
        let label = if ended_on <= today { "Ended" } else { "Ends" };
        pills.push(pill(format!("{label} {}", iso_date(ended_on)), ContractPillTone::Neutral));
        return pills;
    }
    if contract.terms.cancellable_any_time {
        pills.push(pill("Cancellable any time".to_owned(), ContractPillTone::Good));
    } else if let Some(cancel_by) = contract.terms.next_cancel_by(today) {
        if (cancel_by - today).num_days() <= ContractPlanner::LEAD_DAYS {
            pills.push(pill(format!("Cancel by {}", iso_date(cancel_by)), ContractPillTone::Danger));
        } else {
            pills.push(pill(format!("Next cancel-by {}", iso_date(cancel_by)), ContractPillTone::Neutral));
        }
    }
    pills
}

fn detail_of(contract: &Contract, catalog: &Catalog, today: NaiveDate) -> ContractDetailDto {
    let terms = &contract.terms;
    let documents = contract.document_ids.iter().filter(|id| catalog.documents().contains_key(*id)).count();
    let subtitle = [contract.correspondent.clone(), Some(plural(documents, "document"))].into_iter().flatten();
    let filter = contract.correspondent.clone().filter(|name| !text::is_blank(name));
    let shown = filter.as_deref().map_or(0, |name| documents_from(catalog, name));
    let term_end = terms.next_term_end(today);
    ContractDetailDto {
        subtitle: subtitle.collect::<Vec<_>>().join(" · "),
        monthly: contract.price.as_ref().map(|price| money_text(&price.monthly())),
        yearly: contract.price.as_ref().map(|price| money_text(&price.yearly())),
        notice: if terms.cancellable_any_time {
            "Cancellable any time".to_owned()
        } else {
            terms.notice.map_or_else(|| "Not entered".to_owned(), notice_text)
        },
        cancel_by: terms.next_cancel_by(today).map(iso_date),
        term_end: term_end.map(iso_date),
        ended_on: contract.ended_on.map(iso_date),
        history: history_of(contract),
        increase: increase_of(contract, today),
        note: contract.note.clone(),
        document_ids: contract.document_ids.iter().map(ToString::to_string).collect(),
        documents_label: format!("Show {}", plural(shown, "document")),
        documents_filter: filter.filter(|_| shown > 0),
        letter_facts: vec![
            format!("Contract: {}", contract.name),
            match term_end {
                Some(date) => format!("End of the contract: {}", iso_date(date)),
                None => "End of the contract: the next possible date".to_owned(),
            },
        ],
        recipient: contract.correspondent.clone(),
        editor: form_of(contract),
    }
}

fn history_of(contract: &Contract) -> Vec<ContractHistoryPointDto> {
    let mut previous: Option<&ContractPrice> = None;
    contract
        .price_history
        .iter()
        .map(|point| {
            let monthly = point.price.monthly();
            let rose = previous.is_some_and(|before| point.price.costs_more_than(before));
            previous = Some(&point.price);
            ContractHistoryPointDto {
                valid_from: iso_date(point.valid_from),
                label: monthly.amount.to_plain_string(),
                cents: u32::try_from(cents(&monthly.amount)).unwrap_or(0),
                rose,
            }
        })
        .collect()
}

/// "+12 % since 2022": the rise of the monthly price from the first price point to the one valid today.
fn increase_of(contract: &Contract, today: NaiveDate) -> Option<String> {
    let first = contract.price_history.first()?;
    let now = contract.price_on(today)?;
    if first.price.currency != now.currency {
        return None;
    }
    let (from, to) = (cents(&first.price.monthly().amount), cents(&now.monthly().amount));
    if from <= 0 || to <= from {
        return None;
    }
    let percent = ((to - from) * 100 + from / 2) / from;
    Some(format!("+{percent} % since {}", first.valid_from.year()))
}

fn cents(amount: &Decimal) -> i64 {
    amount.times(100).to_i64_truncated()
}

fn documents_from(catalog: &Catalog, correspondent: &str) -> usize {
    let wanted = text::lowercase(text::trim(correspondent));
    catalog
        .documents()
        .values()
        .filter(|document| {
            document.metadata.correspondent.as_deref().is_some_and(|name| text::lowercase(text::trim(name)) == wanted)
        })
        .count()
}

fn money_text(money: &Money) -> String {
    FieldValues::format(&FieldValue::Money { amount: money.amount.clone(), currency: money.currency.clone() })
}

fn plural(count: usize, noun: &str) -> String {
    if count == 1 { format!("1 {noun}") } else { format!("{count} {noun}s") }
}

#[cfg(test)]
mod tests {
    use dms_domain::catalog::{CatalogUpdate, LibraryId};
    use dms_domain::contract::{ContractId, ContractKind, ContractTerms, PricePoint};
    use dms_domain::field::CurrencyCode;
    use dms_domain::money::BillingPeriod;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("date")
    }

    fn today() -> NaiveDate {
        date(2026, 11, 11)
    }

    fn monthly(amount: &str) -> ContractPrice {
        ContractPrice {
            amount: Decimal::parse(amount).expect("decimal"),
            currency: CurrencyCode::new("EUR").expect("currency"),
            period: BillingPeriod::Monthly,
        }
    }

    fn contract(id: &str, name: &str) -> Contract {
        let mut contract =
            Contract::new(ContractId::new(id).expect("id"), name, ContractKind::Telecom).expect("contract");
        contract.correspondent = Some(format!("{name} Ltd"));
        contract
    }

    fn catalog_of(contracts: Vec<Contract>) -> Catalog {
        contracts.into_iter().fold(Catalog::empty(LibraryId::new("library").expect("id")), |catalog, contract| {
            match catalog.save_contract(contract).expect("valid") {
                CatalogUpdate::Applied(catalog) => catalog,
                CatalogUpdate::Rejected(reason) => panic!("{reason:?}"),
            }
        })
    }

    fn internet() -> Contract {
        let mut internet = contract("c-1", "Internet")
            .with_price_point(
                PricePoint { valid_from: date(2022, 1, 1), price: monthly("39.99"), document_id: None },
                today(),
            )
            .with_price_point(
                PricePoint { valid_from: date(2026, 1, 1), price: monthly("49.99"), document_id: None },
                today(),
            );
        internet.terms = ContractTerms {
            term_end: Some(date(2026, 12, 31)),
            renews_by_months: Some(12),
            notice: Some(dms_domain::contract::Notice { count: 1, unit: dms_domain::contract::NoticeUnit::Months }),
            cancellable_any_time: false,
        };
        internet
    }

    #[test]
    fn the_screen_shows_totals_with_their_change_and_cards_with_pills_running_ones_first() {
        let mut gym = contract("c-2", "Gym");
        gym.price = Some(monthly("34.90"));
        gym.ended_on = Some(date(2026, 6, 30));
        let mut power = contract("c-3", "Power");
        power.price = Some(monthly("94.00"));
        power.terms.cancellable_any_time = true;

        let view = contracts_view(&catalog_of(vec![gym, internet(), power]), today());

        let total = &view.totals[0];
        assert_eq!((total.monthly.as_str(), total.yearly.as_str()), ("143.99 EUR", "1727.88 EUR"));
        assert_eq!(total.change.as_deref(), Some("+ 10.00 EUR / month vs. last year"));
        assert!(total.rising);
        let names: Vec<(&str, u32)> = view.cards.iter().map(|card| (card.name.as_str(), card.color)).collect();
        assert_eq!(names, vec![("Internet", 0), ("Power", 1), ("Gym", 2)]);
        let labels = |card: &ContractCardDto| card.pills.iter().map(|pill| pill.label.clone()).collect::<Vec<_>>();
        assert_eq!(labels(&view.cards[0]), vec!["Price ↑".to_owned(), "Cancel by 2026-11-30".to_owned()]);
        assert_eq!(labels(&view.cards[1]), vec!["Cancellable any time".to_owned()]);
        assert_eq!(labels(&view.cards[2]), vec!["Ended 2026-06-30".to_owned()]);
        assert_eq!(view.cards[0].letter, "I");
    }

    #[test]
    fn the_detail_has_the_history_with_rises_the_increase_and_the_letter_facts() {
        let view = contracts_view(&catalog_of(vec![internet()]), today());
        let detail = &view.cards[0].detail;

        assert_eq!(
            detail.history.iter().map(|point| (point.label.as_str(), point.cents, point.rose)).collect::<Vec<_>>(),
            vec![("39.99", 3999, false), ("49.99", 4999, true)]
        );
        assert_eq!(detail.increase.as_deref(), Some("+25 % since 2022"));
        assert_eq!(detail.notice, "1 month");
        assert_eq!(detail.cancel_by.as_deref(), Some("2026-11-30"));
        assert_eq!(
            detail.letter_facts,
            vec!["Contract: Internet".to_owned(), "End of the contract: 2026-12-31".to_owned()]
        );
        assert_eq!(detail.documents_filter, None, "no documents of the correspondent to show");
    }

    #[test]
    fn the_status_card_appears_only_with_a_priced_running_contract() {
        assert!(contracts_status(&catalog_of(vec![contract("c-1", "Unpriced")]), today()).is_none());

        let status = contracts_status(&catalog_of(vec![internet()]), today()).expect("a card");

        assert_eq!(status.monthly, vec!["49.99 EUR".to_owned()]);
        assert_eq!(status.increases_label.as_deref(), Some("1 price increase"));
    }
}
