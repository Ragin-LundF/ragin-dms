use std::collections::HashMap;

use dms_domain::contract::{Contract, ContractId, ContractPrice, ContractTerms, Notice, PricePoint};
use dms_domain::document::DocumentId;
use dms_domain::field::CurrencyCode;
use dms_domain::{iso_date, text};
use indexmap::IndexMap;

use crate::catalog::dto::contract::{ContractDto, ContractPriceDto, ContractTermsDto, PricePointDto};
use crate::catalog::mapper::{field_mapper, unknown_fields};
use crate::enum_names::{self, BILLING_PERIODS, CONTRACT_KINDS, NOTICE_UNITS};
use crate::yaml::YamlError;

use super::number;

/// The contracts sorted by id; fields only newer versions know are taken over from `previous` per id.
pub fn to_dto(contracts: &IndexMap<ContractId, Contract>, previous: &[ContractDto]) -> Vec<ContractDto> {
    let previous_by_id: HashMap<&str, &ContractDto> =
        previous.iter().map(|contract| (contract.id.as_str(), contract)).collect();
    let mut sorted: Vec<&Contract> = contracts.values().collect();
    sorted.sort_by(|first, second| text::compare(first.id.value(), second.id.value()));
    sorted
        .into_iter()
        .map(|contract| {
            let previous = previous_by_id.get(contract.id.value()).copied();
            let mut dto = contract_to_dto(contract, previous.and_then(|previous| previous.terms.as_ref()));
            unknown_fields::carry(&mut dto, previous);
            dto
        })
        .collect()
}

pub fn to_domain(dtos: &[ContractDto]) -> Result<IndexMap<ContractId, Contract>, YamlError> {
    dtos.iter()
        .map(|dto| {
            let contract = contract_to_domain(dto)?;
            Ok((contract.id.clone(), contract))
        })
        .collect()
}

fn contract_to_dto(contract: &Contract, previous_terms: Option<&ContractTermsDto>) -> ContractDto {
    ContractDto {
        id: contract.id.value().to_owned(),
        name: contract.name.clone(),
        correspondent: contract.correspondent.clone(),
        kind: enum_names::name_of(CONTRACT_KINDS, contract.kind).to_owned(),
        price: contract.price.as_ref().map(price_to_dto),
        price_history: contract
            .price_history
            .iter()
            .map(|point| PricePointDto {
                valid_from: iso_date(point.valid_from),
                price: price_to_dto(&point.price),
                document_id: point.document_id.as_ref().map(|id| id.value().to_owned()),
            })
            .collect(),
        terms: terms_to_dto(&contract.terms, previous_terms),
        document_ids: contract.document_ids.iter().map(|id| id.value().to_owned()).collect(),
        ended_on: contract.ended_on.map(iso_date),
        note: contract.note.clone(),
        unknown: IndexMap::new(),
    }
}

/// `None` for empty terms, unless the previous file holds fields of a newer version there.
fn terms_to_dto(terms: &ContractTerms, previous: Option<&ContractTermsDto>) -> Option<ContractTermsDto> {
    if *terms == ContractTerms::default() && !unknown_fields::has_any(previous) {
        return None;
    }
    let mut dto = ContractTermsDto {
        term_end: terms.term_end.map(iso_date),
        renews_by_months: terms.renews_by_months.map(i64::from),
        notice_count: terms.notice.map(|notice| i64::from(notice.count)),
        notice_unit: terms.notice.map(|notice| enum_names::name_of(NOTICE_UNITS, notice.unit).to_owned()),
        cancellable_any_time: terms.cancellable_any_time,
        unknown: IndexMap::new(),
    };
    unknown_fields::carry(&mut dto, previous);
    Some(dto)
}

fn price_to_dto(price: &ContractPrice) -> ContractPriceDto {
    ContractPriceDto {
        amount: price.amount.to_plain_string(),
        currency: price.currency.value().to_owned(),
        period: enum_names::name_of(BILLING_PERIODS, price.period).to_owned(),
    }
}

fn contract_to_domain(dto: &ContractDto) -> Result<Contract, YamlError> {
    Ok(Contract {
        id: ContractId::new(dto.id.as_str())?,
        name: dto.name.clone(),
        correspondent: dto.correspondent.clone(),
        kind: enum_names::parse(CONTRACT_KINDS, &dto.kind)?,
        price: dto.price.as_ref().map(price_to_domain).transpose()?,
        price_history: dto
            .price_history
            .iter()
            .map(|point| {
                Ok(PricePoint {
                    valid_from: field_mapper::date_of(&point.valid_from)?,
                    price: price_to_domain(&point.price)?,
                    document_id: point.document_id.as_deref().map(DocumentId::new).transpose()?,
                })
            })
            .collect::<Result<_, YamlError>>()?,
        terms: dto.terms.as_ref().map(terms_to_domain).transpose()?.unwrap_or_default(),
        document_ids: dto.document_ids.iter().map(|id| DocumentId::new(id.as_str())).collect::<Result<_, _>>()?,
        ended_on: dto.ended_on.as_deref().map(field_mapper::date_of).transpose()?,
        note: dto.note.clone(),
    })
}

fn terms_to_domain(dto: &ContractTermsDto) -> Result<ContractTerms, YamlError> {
    let notice = match (dto.notice_count, dto.notice_unit.as_deref()) {
        (Some(count), Some(unit)) => {
            Some(Notice { count: number(count, "Notice")?, unit: enum_names::parse(NOTICE_UNITS, unit)? })
        }
        (None, None) => None,
        _ => return Err(YamlError("A notice needs a count and a unit".to_owned())),
    };
    Ok(ContractTerms {
        term_end: dto.term_end.as_deref().map(field_mapper::date_of).transpose()?,
        renews_by_months: dto.renews_by_months.map(|months| number(months, "Renewal months")).transpose()?,
        notice,
        cancellable_any_time: dto.cancellable_any_time,
    })
}

fn price_to_domain(dto: &ContractPriceDto) -> Result<ContractPrice, YamlError> {
    Ok(ContractPrice {
        amount: field_mapper::decimal(&dto.amount)?,
        currency: CurrencyCode::new(dto.currency.as_str())?,
        period: enum_names::parse(BILLING_PERIODS, &dto.period)?,
    })
}
