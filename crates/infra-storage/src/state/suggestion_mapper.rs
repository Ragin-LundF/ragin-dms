//! Pending suggestions in `tasks.yaml`. Like the Kotlin version, a suggested target folder, its reason and the rule
//! notes are not stored; field values, the parts read from an e-invoice, the suggested tax assignment and the actions
//! a letter requests are (2.1, so a suggested amount, due date, IBAN, tax year or follow-up reaches the review).

use dms_domain::classification::{
    ClassificationSuggestion, EInvoicePart, RequestedAction, SuggestedCategory, SuggestedTag,
};
use dms_domain::document::DocumentId;
use dms_domain::field::FieldKey;
use dms_domain::iso_date;
use dms_domain::taxonomy::{CategoryId, TagName};

use crate::catalog::dto::owned;
use crate::catalog::mapper::field_mapper;
use crate::catalog::mapper::records::tax_mapper;
use crate::state::dto::{RequestedActionDto, SuggestedTagDto, SuggestionDto};
use crate::yaml::YamlError;

pub fn to_dto(suggestion: &ClassificationSuggestion) -> SuggestionDto {
    let (existing, new_name, new_parent) = match &suggestion.category {
        Some(SuggestedCategory::Existing(id)) => (Some(id.value().to_owned()), None, None),
        Some(SuggestedCategory::New { name, parent_id }) => {
            (None, Some(name.clone()), parent_id.as_ref().map(|parent| parent.value().to_owned()))
        }
        None => (None, None, None),
    };
    SuggestionDto {
        document_id: suggestion.document_id.value().to_owned(),
        model: suggestion.model.clone(),
        title: suggestion.title.clone(),
        existing_category_id: existing,
        new_category_name: new_name,
        new_category_parent_id: new_parent,
        tags: suggestion
            .tags
            .iter()
            .map(|tag| SuggestedTagDto { name: tag.name.value().to_owned(), is_new: tag.is_new })
            .collect(),
        document_date: suggestion.document_date.map(iso_date),
        correspondent: suggestion.correspondent.clone(),
        file_name: suggestion.file_name.clone(),
        transcription: suggestion.transcription.clone(),
        confidence: suggestion.confidence,
        tax: suggestion.tax.as_ref().map(|tax| tax_mapper::assignment_to_dto(tax, None)),
        fields: field_mapper::values_to_dto(&suggestion.fields),
        e_invoice: suggestion.e_invoice.iter().map(part_name).collect(),
        requested_actions: suggestion
            .requested_actions
            .iter()
            .map(|action| RequestedActionDto {
                text: action.text.clone(),
                due_date: action.due_date.map(iso_date),
                references: action.references.clone(),
            })
            .collect(),
    }
}

pub fn to_domain(dto: &SuggestionDto) -> Result<ClassificationSuggestion, YamlError> {
    let mut suggestion =
        ClassificationSuggestion::empty(DocumentId::new(dto.document_id.as_str())?, dto.model.as_str());
    suggestion.title.clone_from(&dto.title);
    suggestion.category = category_of(dto)?;
    suggestion.tags = dto
        .tags
        .iter()
        .map(|tag| Ok(SuggestedTag { name: TagName::of(&tag.name)?, is_new: tag.is_new }))
        .collect::<Result<_, YamlError>>()?;
    suggestion.document_date = dto.document_date.as_deref().map(date_of).transpose()?;
    suggestion.correspondent = owned(dto.correspondent.as_deref());
    suggestion.file_name.clone_from(&dto.file_name);
    suggestion.transcription.clone_from(&dto.transcription);
    suggestion.confidence = dto.confidence;
    suggestion.tax = dto.tax.as_ref().map(tax_mapper::assignment_to_domain).transpose()?;
    suggestion.fields = field_mapper::values_to_domain(&dto.fields)?;
    suggestion.e_invoice = dto.e_invoice.iter().map(|name| part_of(name)).collect::<Result<_, _>>()?;
    suggestion.requested_actions = dto
        .requested_actions
        .iter()
        .map(|action| {
            Ok(RequestedAction {
                text: action.text.clone(),
                due_date: action.due_date.as_deref().map(date_of).transpose()?,
                references: action.references.clone(),
            })
        })
        .collect::<Result<_, YamlError>>()?;
    suggestion.validate()?;
    Ok(suggestion)
}

fn category_of(dto: &SuggestionDto) -> Result<Option<SuggestedCategory>, YamlError> {
    if let Some(existing) = &dto.existing_category_id {
        return Ok(Some(SuggestedCategory::Existing(CategoryId::new(existing.as_str())?)));
    }
    let Some(name) = &dto.new_category_name else { return Ok(None) };
    let parent_id = dto.new_category_parent_id.as_deref().map(CategoryId::new).transpose()?;
    Ok(Some(SuggestedCategory::New { name: name.clone(), parent_id }))
}

fn date_of(text: &str) -> Result<chrono::NaiveDate, YamlError> {
    dms_domain::parse_iso_date(text).ok_or_else(|| YamlError(format!("Text '{text}' could not be parsed as a date")))
}

const FIELD_PART: &str = "field:";

fn part_name(part: &EInvoicePart) -> String {
    match part {
        EInvoicePart::Correspondent => "correspondent".to_owned(),
        EInvoicePart::DocumentDate => "documentDate".to_owned(),
        EInvoicePart::Field(key) => format!("{FIELD_PART}{key}"),
    }
}

fn part_of(name: &str) -> Result<EInvoicePart, YamlError> {
    match name {
        "correspondent" => Ok(EInvoicePart::Correspondent),
        "documentDate" => Ok(EInvoicePart::DocumentDate),
        _ => match name.strip_prefix(FIELD_PART) {
            Some(key) => Ok(EInvoicePart::Field(FieldKey::new(key)?)),
            None => Err(YamlError(format!("Unknown e-invoice part: {name}"))),
        },
    }
}
