use std::collections::HashMap;

use dms_domain::cases::{Case, CaseEntry, CaseId, CaseNote, FollowUp, FollowUpId, FollowUpSource};
use dms_domain::document::DocumentId;
use dms_domain::{iso_date, text};
use indexmap::IndexMap;

use crate::catalog::dto::case::{CaseDto, CaseEntryDto, CaseNoteDto, FollowUpDto};
use crate::catalog::mapper::{field_mapper, unknown_fields};
use crate::enum_names::{self, CASE_STATUSES, ENTRY_DIRECTIONS};
use crate::yaml::YamlError;

use super::number;

const MANUAL: &str = "manual";
const AI: &str = "ai";

/// The cases sorted by id; fields only newer versions know are taken over from `previous` per case and follow-up.
pub fn to_dto(cases: &IndexMap<CaseId, Case>, previous: &[CaseDto]) -> Vec<CaseDto> {
    let previous_by_id: HashMap<&str, &CaseDto> = previous.iter().map(|case| (case.id.as_str(), case)).collect();
    let mut sorted: Vec<&Case> = cases.values().collect();
    sorted.sort_by(|first, second| text::compare(first.id.value(), second.id.value()));
    sorted
        .into_iter()
        .map(|case| {
            let previous = previous_by_id.get(case.id.value()).copied();
            let mut dto = case_to_dto(case);
            if let Some(previous) = previous {
                unknown_fields::carry_all(&mut dto.follow_ups, &previous.follow_ups, |follow_up| follow_up.id.clone());
            }
            unknown_fields::carry(&mut dto, previous);
            dto
        })
        .collect()
}

pub fn to_domain(dtos: &[CaseDto]) -> Result<IndexMap<CaseId, Case>, YamlError> {
    dtos.iter()
        .map(|dto| {
            let case = case_to_domain(dto)?;
            Ok((case.id.clone(), case))
        })
        .collect()
}

fn case_to_dto(case: &Case) -> CaseDto {
    CaseDto {
        id: case.id.value().to_owned(),
        title: case.title.clone(),
        correspondent: case.correspondent.clone(),
        status: enum_names::name_of(CASE_STATUSES, case.status).to_owned(),
        opened_on: iso_date(case.opened_on),
        closed_on: case.closed_on.map(iso_date),
        waiting_since: case.waiting_since.map(iso_date),
        remind_after_days: i64::from(case.remind_after_days),
        entries: case
            .entries
            .iter()
            .map(|entry| CaseEntryDto {
                document_id: entry.document_id.value().to_owned(),
                direction: enum_names::name_of(ENTRY_DIRECTIONS, entry.direction).to_owned(),
                date: iso_date(entry.date),
            })
            .collect(),
        notes: case
            .notes
            .iter()
            .map(|note| CaseNoteDto { date: iso_date(note.date), text: note.text.clone() })
            .collect(),
        follow_ups: case.follow_ups.iter().map(follow_up_to_dto).collect(),
        unknown: IndexMap::new(),
    }
}

fn follow_up_to_dto(follow_up: &FollowUp) -> FollowUpDto {
    let (source, source_document_id) = match &follow_up.source {
        FollowUpSource::Manual => (MANUAL, None),
        FollowUpSource::Ai { document_id } => (AI, Some(document_id.value().to_owned())),
    };
    FollowUpDto {
        id: follow_up.id.value().to_owned(),
        date: iso_date(follow_up.date),
        text: follow_up.text.clone(),
        done: follow_up.done,
        source: source.to_owned(),
        source_document_id,
        unknown: IndexMap::new(),
    }
}

fn case_to_domain(dto: &CaseDto) -> Result<Case, YamlError> {
    Ok(Case {
        id: CaseId::new(dto.id.as_str())?,
        title: dto.title.clone(),
        correspondent: dto.correspondent.clone(),
        status: enum_names::parse(CASE_STATUSES, &dto.status)?,
        opened_on: field_mapper::date_of(&dto.opened_on)?,
        closed_on: dto.closed_on.as_deref().map(field_mapper::date_of).transpose()?,
        waiting_since: dto.waiting_since.as_deref().map(field_mapper::date_of).transpose()?,
        remind_after_days: number(dto.remind_after_days, "Reminder days")?,
        entries: dto
            .entries
            .iter()
            .map(|entry| {
                Ok(CaseEntry {
                    document_id: DocumentId::new(entry.document_id.as_str())?,
                    direction: enum_names::parse(ENTRY_DIRECTIONS, &entry.direction)?,
                    date: field_mapper::date_of(&entry.date)?,
                })
            })
            .collect::<Result<_, YamlError>>()?,
        notes: dto
            .notes
            .iter()
            .map(|note| Ok(CaseNote { date: field_mapper::date_of(&note.date)?, text: note.text.clone() }))
            .collect::<Result<_, YamlError>>()?,
        follow_ups: dto.follow_ups.iter().map(follow_up_to_domain).collect::<Result<_, _>>()?,
    })
}

fn follow_up_to_domain(dto: &FollowUpDto) -> Result<FollowUp, YamlError> {
    let source = match (dto.source.as_str(), dto.source_document_id.as_deref()) {
        (AI, Some(document_id)) => FollowUpSource::Ai { document_id: DocumentId::new(document_id)? },
        (MANUAL, _) => FollowUpSource::Manual,
        (other, _) => return Err(YamlError(format!("Unknown follow-up source: {other}"))),
    };
    Ok(FollowUp {
        id: FollowUpId::new(dto.id.as_str())?,
        date: field_mapper::date_of(&dto.date)?,
        text: dto.text.clone(),
        done: dto.done,
        source,
    })
}
