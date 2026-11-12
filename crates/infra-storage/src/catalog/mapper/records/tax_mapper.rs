use std::collections::HashMap;

use dms_domain::document::DocumentId;
use dms_domain::iso_date;
use dms_domain::tax::{TaxAssignment, TaxChecklistItem, TaxItemId, TaxYear};
use indexmap::IndexMap;

use crate::catalog::dto::tax::{TaxAssignmentDto, TaxItemDto, TaxYearDto};
use crate::catalog::mapper::{field_mapper, unknown_fields};
use crate::enum_names::{self, TAX_AREAS, TAX_ITEM_STATES};
use crate::yaml::YamlError;

use super::{money_to_domain, money_to_dto, number};

/// The tax years by year; fields only newer versions know are taken over from `previous` per year and item.
pub fn years_to_dto(years: &IndexMap<i32, TaxYear>, previous: &[TaxYearDto]) -> Vec<TaxYearDto> {
    let previous_by_year: HashMap<i64, &TaxYearDto> = previous.iter().map(|year| (year.year, year)).collect();
    let mut sorted: Vec<&TaxYear> = years.values().collect();
    sorted.sort_by_key(|year| year.year);
    sorted
        .into_iter()
        .map(|year| {
            let previous = previous_by_year.get(&i64::from(year.year)).copied();
            let mut dto = TaxYearDto {
                year: i64::from(year.year),
                items: year.items.iter().map(item_to_dto).collect(),
                sent_on: year.sent_on.map(iso_date),
                unknown: IndexMap::new(),
            };
            if let Some(previous) = previous {
                unknown_fields::carry_all(&mut dto.items, &previous.items, |item| item.id.clone());
            }
            unknown_fields::carry(&mut dto, previous);
            dto
        })
        .collect()
}

pub fn years_to_domain(dtos: &[TaxYearDto]) -> Result<IndexMap<i32, TaxYear>, YamlError> {
    dtos.iter()
        .map(|dto| {
            let year: i32 = number(dto.year, "Tax year")?;
            let tax_year = TaxYear {
                year,
                items: dto.items.iter().map(item_to_domain).collect::<Result<_, _>>()?,
                sent_on: dto.sent_on.as_deref().map(field_mapper::date_of).transpose()?,
            };
            Ok((year, tax_year))
        })
        .collect()
}

pub fn assignment_to_dto(tax: &TaxAssignment, previous: Option<&TaxAssignmentDto>) -> TaxAssignmentDto {
    let mut dto = TaxAssignmentDto {
        year: i64::from(tax.year),
        area: enum_names::name_of(TAX_AREAS, tax.area).to_owned(),
        deductible: tax.deductible.as_ref().map(money_to_dto),
        unknown: IndexMap::new(),
    };
    unknown_fields::carry(&mut dto, previous);
    dto
}

pub fn assignment_to_domain(dto: &TaxAssignmentDto) -> Result<TaxAssignment, YamlError> {
    Ok(TaxAssignment {
        year: number(dto.year, "Tax year")?,
        area: enum_names::parse(TAX_AREAS, &dto.area)?,
        deductible: dto.deductible.as_ref().map(money_to_domain).transpose()?,
    })
}

fn item_to_dto(item: &TaxChecklistItem) -> TaxItemDto {
    TaxItemDto {
        id: item.id.value().to_owned(),
        area: enum_names::name_of(TAX_AREAS, item.area).to_owned(),
        label: item.label.clone(),
        correspondent: item.correspondent.clone(),
        document_id: item.document_id.as_ref().map(|id| id.value().to_owned()),
        state: enum_names::name_of(TAX_ITEM_STATES, item.state).to_owned(),
        note: item.note.clone(),
        added_by_user: item.added_by_user,
        unknown: IndexMap::new(),
    }
}

fn item_to_domain(dto: &TaxItemDto) -> Result<TaxChecklistItem, YamlError> {
    Ok(TaxChecklistItem {
        id: TaxItemId::new(dto.id.as_str())?,
        area: enum_names::parse(TAX_AREAS, &dto.area)?,
        label: dto.label.clone(),
        correspondent: dto.correspondent.clone(),
        document_id: dto.document_id.as_deref().map(DocumentId::new).transpose()?,
        state: enum_names::parse(TAX_ITEM_STATES, &dto.state)?,
        note: dto.note.clone(),
        added_by_user: dto.added_by_user,
    })
}
