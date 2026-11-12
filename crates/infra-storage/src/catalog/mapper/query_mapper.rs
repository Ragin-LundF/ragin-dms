//! Saved searches and their queries. Only the conditions a query sets are written.

use dms_domain::field::FieldKey;
use dms_domain::iso_date;
use dms_domain::library::RelativePath;
use dms_domain::query::{DateRange, DocumentQuery, FieldCondition, SavedSearch, SavedSearchId, TagFilter};
use dms_domain::tax::TaxFilter;
use dms_domain::taxonomy::{CategoryId, TagName};
use indexmap::{IndexMap, IndexSet};

use crate::catalog::dto::settings::{FieldConditionDto, QueryDto, SavedSearchDto};
use crate::catalog::mapper::field_mapper;
use crate::enum_names::{self, FIELD_COMPARISONS, TAX_AREAS};
use crate::instant;
use crate::yaml::YamlError;

pub fn search_to_dto(search: &SavedSearch) -> SavedSearchDto {
    SavedSearchDto {
        id: search.id.value().to_owned(),
        name: search.name.clone(),
        query: query_to_dto(&search.query),
        unknown: IndexMap::new(),
    }
}

pub fn search_to_domain(search: &SavedSearchDto) -> Result<SavedSearch, YamlError> {
    Ok(SavedSearch {
        id: SavedSearchId::new(search.id.as_str())?,
        name: search.name.clone(),
        query: query_to_domain(&search.query)?,
    })
}

pub fn query_to_dto(query: &DocumentQuery) -> QueryDto {
    QueryDto {
        text: query.text.clone(),
        category: query.category.as_ref().map(|category| category.value().to_owned()),
        uncategorized: query.uncategorized.then_some(true),
        tags_all: sorted_tags(&query.tags.all),
        tags_any: sorted_tags(&query.tags.any),
        tags_none: sorted_tags(&query.tags.none),
        untagged: query.untagged.then_some(true),
        from: query.dates.as_ref().and_then(|dates| dates.from).map(iso_date),
        to: query.dates.as_ref().and_then(|dates| dates.to).map(iso_date),
        correspondent: query.correspondent.clone(),
        folder: query.folder.as_ref().map(|folder| folder.value().to_owned()),
        fields: query
            .fields
            .iter()
            .map(|condition| FieldConditionDto {
                key: condition.key.value().to_owned(),
                comparison: enum_names::name_of(FIELD_COMPARISONS, condition.comparison).to_owned(),
                value: condition.value.as_ref().map(field_mapper::value_to_dto),
            })
            .collect(),
        added_since: query.added_since.map(instant::format),
        content_text: query.content_text.clone(),
        tax_year: query.tax.as_ref().and_then(|tax| tax.year),
        tax_area: query
            .tax
            .as_ref()
            .and_then(|tax| tax.area)
            .map(|area| enum_names::name_of(TAX_AREAS, area).to_owned()),
        tax_tag_fallback: query
            .tax
            .as_ref()
            .and_then(|tax| tax.tag_fallback.as_ref())
            .map(|tag| tag.value().to_owned()),
        unknown: IndexMap::new(),
    }
}

pub fn query_to_domain(query: &QueryDto) -> Result<DocumentQuery, YamlError> {
    let from = query.from.as_deref().map(field_mapper::date_of).transpose()?;
    let to = query.to.as_deref().map(field_mapper::date_of).transpose()?;
    let dates = if from.is_none() && to.is_none() { None } else { Some(DateRange::new(from, to)?) };
    Ok(DocumentQuery {
        text: query.text.clone(),
        category: query.category.as_deref().map(CategoryId::new).transpose()?,
        uncategorized: query.uncategorized == Some(true),
        tags: TagFilter {
            all: tags_of(&query.tags_all)?,
            any: tags_of(&query.tags_any)?,
            none: tags_of(&query.tags_none)?,
        },
        untagged: query.untagged == Some(true),
        dates,
        correspondent: query.correspondent.clone(),
        folder: query.folder.as_deref().map(RelativePath::of).transpose()?,
        fields: query
            .fields
            .iter()
            .map(|condition| {
                let value = condition.value.as_ref().map(field_mapper::value_to_domain).transpose()?;
                let comparison = enum_names::parse(FIELD_COMPARISONS, &condition.comparison)?;
                Ok(FieldCondition::new(FieldKey::new(condition.key.as_str())?, comparison, value)?)
            })
            .collect::<Result<_, YamlError>>()?,
        added_since: query.added_since.as_deref().map(instant::parse).transpose()?,
        content_text: query.content_text.clone(),
        tax: tax_of(query)?,
    })
}

/// The tax filter; `None` when the query sets none of its keys.
fn tax_of(query: &QueryDto) -> Result<Option<TaxFilter>, YamlError> {
    if query.tax_year.is_none() && query.tax_area.is_none() && query.tax_tag_fallback.is_none() {
        return Ok(None);
    }
    Ok(Some(TaxFilter {
        year: query.tax_year,
        area: query.tax_area.as_deref().map(|area| enum_names::parse(TAX_AREAS, area)).transpose()?,
        tag_fallback: query.tax_tag_fallback.as_deref().map(TagName::of).transpose()?,
    }))
}

pub fn sorted_tags(tags: &IndexSet<TagName>) -> Vec<String> {
    let mut sorted: Vec<&TagName> = tags.iter().collect();
    sorted.sort();
    sorted.into_iter().map(|tag| tag.value().to_owned()).collect()
}

pub fn tags_of(names: &[String]) -> Result<IndexSet<TagName>, YamlError> {
    names.iter().map(|name| Ok(TagName::of(name)?)).collect()
}
