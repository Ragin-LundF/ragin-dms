//! Custom field definitions and values. Numbers are written as plain decimal strings, never as floats.

use dms_domain::field::{CurrencyCode, Decimal, FieldDefinition, FieldKey, FieldValue};
use dms_domain::{iso_date, parse_iso_date, text};
use indexmap::IndexMap;

use crate::catalog::dto::FieldValueDto;
use crate::catalog::dto::settings::FieldDefinitionDto;
use crate::enum_names::{self, FIELD_ROLES, FIELD_TYPES};
use crate::yaml::YamlError;

pub fn definition_to_dto(definition: &FieldDefinition) -> FieldDefinitionDto {
    FieldDefinitionDto {
        key: definition.key.value().to_owned(),
        label: definition.label.clone(),
        field_type: enum_names::name_of(FIELD_TYPES, definition.field_type).to_owned(),
        role: definition.role.map(|role| enum_names::name_of(FIELD_ROLES, role).to_owned()),
        unknown: IndexMap::new(),
    }
}

pub fn definition_to_domain(definition: &FieldDefinitionDto) -> Result<FieldDefinition, YamlError> {
    Ok(FieldDefinition {
        key: FieldKey::new(definition.key.as_str())?,
        label: definition.label.clone(),
        field_type: enum_names::parse(FIELD_TYPES, &definition.field_type)?,
        role: definition.role.as_deref().map(|role| enum_names::parse(FIELD_ROLES, role)).transpose()?,
    })
}

/// The values sorted by key, so the file is stable.
pub fn values_to_dto(values: &IndexMap<FieldKey, FieldValue>) -> IndexMap<String, FieldValueDto> {
    let mut sorted: Vec<(&FieldKey, &FieldValue)> = values.iter().collect();
    sorted.sort_by(|(first, _), (second, _)| text::compare(first.value(), second.value()));
    sorted.into_iter().map(|(key, value)| (key.value().to_owned(), value_to_dto(value))).collect()
}

pub fn values_to_domain(values: &IndexMap<String, FieldValueDto>) -> Result<IndexMap<FieldKey, FieldValue>, YamlError> {
    values.iter().map(|(key, value)| Ok((FieldKey::new(key.as_str())?, value_to_domain(value)?))).collect()
}

pub fn value_to_dto(value: &FieldValue) -> FieldValueDto {
    match value {
        FieldValue::Text(text) => FieldValueDto { text: Some(text.clone()), ..FieldValueDto::default() },
        FieldValue::Number(number) => {
            FieldValueDto { number: Some(number.to_plain_string()), ..FieldValueDto::default() }
        }
        FieldValue::Money { amount, currency } => FieldValueDto {
            amount: Some(amount.to_plain_string()),
            currency: Some(currency.value().to_owned()),
            ..FieldValueDto::default()
        },
        FieldValue::Date(date) => FieldValueDto { date: Some(iso_date(*date)), ..FieldValueDto::default() },
        FieldValue::Flag(flag) => FieldValueDto { flag: Some(*flag), ..FieldValueDto::default() },
    }
}

/// # Errors
/// Fails for a value that sets no or more than one type, or an unreadable number, amount or date.
pub fn value_to_domain(value: &FieldValueDto) -> Result<FieldValue, YamlError> {
    let mut candidates = Vec::new();
    if let Some(text) = &value.text {
        candidates.push(FieldValue::Text(text.clone()));
    }
    if let Some(number) = &value.number {
        candidates.push(FieldValue::Number(decimal(number)?));
    }
    if let Some(amount) = &value.amount {
        let currency = value.currency.as_deref().ok_or_else(|| YamlError("Amount without currency".to_owned()))?;
        candidates.push(FieldValue::Money { amount: decimal(amount)?, currency: CurrencyCode::new(currency)? });
    }
    if let Some(date) = &value.date {
        candidates.push(FieldValue::Date(date_of(date)?));
    }
    if let Some(flag) = value.flag {
        candidates.push(FieldValue::Flag(flag));
    }
    match <[FieldValue; 1]>::try_from(candidates) {
        Ok([single]) => Ok(single),
        Err(_) => Err(YamlError("A field value must have exactly one type".to_owned())),
    }
}

/// `LocalDate.parse`.
pub fn date_of(text: &str) -> Result<chrono::NaiveDate, YamlError> {
    parse_iso_date(text).ok_or_else(|| YamlError(format!("Text '{text}' could not be parsed as a date")))
}

pub fn decimal(text: &str) -> Result<Decimal, YamlError> {
    Decimal::parse(text).ok_or_else(|| YamlError(format!("Not a number: {text}")))
}
