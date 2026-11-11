//! Pending suggestions in `tasks.yaml`. Like the Kotlin version, only these fields are kept: a suggested target
//! folder, its reason, field values and rule notes are not stored.

use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory, SuggestedTag};
use dms_domain::document::DocumentId;
use dms_domain::iso_date;
use dms_domain::taxonomy::{CategoryId, TagName};

use crate::catalog::dto::owned;
use crate::state::dto::{SuggestedTagDto, SuggestionDto};
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
