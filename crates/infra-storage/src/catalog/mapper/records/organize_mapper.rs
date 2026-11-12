use dms_domain::contract::ContractId;
use dms_domain::document::{ContentHash, DocumentId};
use dms_domain::emergency::{EmergencyEntry, EmergencyFolder, EmergencyItem, EmergencyItemId};
use dms_domain::expectation::ExpectationOverride;
use dms_domain::iso_date;
use dms_domain::settings::OwnerDetails;
use dms_domain::taxonomy::CategoryId;
use indexmap::IndexMap;

use crate::catalog::dto::organize::{
    EmergencyContactDto, EmergencyFolderDto, EmergencyItemDto, ExpectationDto, OwnerDto,
};
use crate::catalog::mapper::{field_mapper, unknown_fields};
use crate::enum_names::{self, EMERGENCY_GROUPS, EXPECTATION_MODES};
use crate::yaml::YamlError;

use super::number;

pub fn owner_to_dto(owner: &OwnerDetails, previous: Option<&OwnerDto>) -> OwnerDto {
    let mut dto = OwnerDto {
        name: owner.name.clone(),
        address_lines: owner.address_lines.clone(),
        email: owner.email.clone(),
        phone: owner.phone.clone(),
        unknown: IndexMap::new(),
    };
    unknown_fields::carry(&mut dto, previous);
    dto
}

pub fn owner_to_domain(dto: &OwnerDto) -> OwnerDetails {
    OwnerDetails {
        name: dto.name.clone(),
        address_lines: dto.address_lines.clone(),
        email: dto.email.clone(),
        phone: dto.phone.clone(),
    }
}

/// The overrides in the user's order; fields only newer versions know are taken over per sender and category.
pub fn expectations_to_dto(overrides: &[ExpectationOverride], previous: &[ExpectationDto]) -> Vec<ExpectationDto> {
    let mut dtos: Vec<ExpectationDto> = overrides
        .iter()
        .map(|item| ExpectationDto {
            correspondent: item.correspondent.clone(),
            category_id: item.category_id.as_ref().map(|id| id.value().to_owned()),
            mode: enum_names::name_of(EXPECTATION_MODES, item.mode).to_owned(),
            unknown: IndexMap::new(),
        })
        .collect();
    unknown_fields::carry_all(&mut dtos, previous, |dto| {
        (dms_domain::text::lowercase(dms_domain::text::trim(&dto.correspondent)), dto.category_id.clone())
    });
    dtos
}

pub fn expectations_to_domain(dtos: &[ExpectationDto]) -> Result<Vec<ExpectationOverride>, YamlError> {
    dtos.iter()
        .map(|dto| {
            Ok(ExpectationOverride {
                correspondent: dto.correspondent.clone(),
                category_id: dto.category_id.as_deref().map(CategoryId::new).transpose()?,
                mode: enum_names::parse(EXPECTATION_MODES, &dto.mode)?,
            })
        })
        .collect()
}

pub fn emergency_to_dto(folder: &EmergencyFolder, previous: Option<&EmergencyFolderDto>) -> EmergencyFolderDto {
    let mut dto = EmergencyFolderDto {
        items: folder.items.iter().map(item_to_dto).collect(),
        last_refreshed: folder.last_refreshed.map(iso_date),
        refresh_months: i64::from(folder.refresh_months),
        intro_text: folder.intro_text.clone(),
        unknown: IndexMap::new(),
    };
    if let Some(previous) = previous {
        unknown_fields::carry_all(&mut dto.items, &previous.items, |item| item.id.clone());
    }
    unknown_fields::carry(&mut dto, previous);
    dto
}

pub fn emergency_to_domain(dto: &EmergencyFolderDto) -> Result<EmergencyFolder, YamlError> {
    Ok(EmergencyFolder {
        items: dto.items.iter().map(item_to_domain).collect::<Result<_, _>>()?,
        last_refreshed: dto.last_refreshed.as_deref().map(field_mapper::date_of).transpose()?,
        refresh_months: number(dto.refresh_months, "Refresh months")?,
        intro_text: dto.intro_text.clone(),
    })
}

fn item_to_dto(item: &EmergencyItem) -> EmergencyItemDto {
    let mut dto = EmergencyItemDto {
        id: item.id.value().to_owned(),
        group: enum_names::name_of(EMERGENCY_GROUPS, item.group).to_owned(),
        document_id: None,
        contract_id: None,
        contact: None,
        note: None,
        included: item.included,
        recorded_hash: item.recorded_hash.as_ref().map(|hash| hash.value().to_owned()),
        unknown: IndexMap::new(),
    };
    match &item.entry {
        EmergencyEntry::Document(id) => dto.document_id = Some(id.value().to_owned()),
        EmergencyEntry::Contract(id) => dto.contract_id = Some(id.value().to_owned()),
        EmergencyEntry::Contact { name, role, phone, email, note } => {
            dto.contact = Some(EmergencyContactDto {
                name: name.clone(),
                role: role.clone(),
                phone: phone.clone(),
                email: email.clone(),
                note: note.clone(),
            });
        }
        EmergencyEntry::Note(text) => dto.note = Some(text.clone()),
    }
    dto
}

fn item_to_domain(dto: &EmergencyItemDto) -> Result<EmergencyItem, YamlError> {
    let entry = match (&dto.document_id, &dto.contract_id, &dto.contact, &dto.note) {
        (Some(id), None, None, None) => EmergencyEntry::Document(DocumentId::new(id.as_str())?),
        (None, Some(id), None, None) => EmergencyEntry::Contract(ContractId::new(id.as_str())?),
        (None, None, Some(contact), None) => EmergencyEntry::Contact {
            name: contact.name.clone(),
            role: contact.role.clone(),
            phone: contact.phone.clone(),
            email: contact.email.clone(),
            note: contact.note.clone(),
        },
        (None, None, None, Some(text)) => EmergencyEntry::Note(text.clone()),
        _ => return Err(YamlError("An emergency item must hold exactly one entry".to_owned())),
    };
    Ok(EmergencyItem {
        id: EmergencyItemId::new(dto.id.as_str())?,
        group: enum_names::parse(EMERGENCY_GROUPS, &dto.group)?,
        entry,
        included: dto.included,
        recorded_hash: dto.recorded_hash.as_deref().map(ContentHash::from_hex).transpose()?,
    })
}
