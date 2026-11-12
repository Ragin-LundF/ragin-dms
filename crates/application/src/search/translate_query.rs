use std::sync::Arc;

use dms_domain::catalog::Catalog;
use dms_domain::field::FieldValues;
use dms_domain::query::{DateRange, DocumentQuery, FieldCondition};
use dms_domain::tax::TaxArea;
use dms_domain::taxonomy::{CategoryId, TagName};
use dms_domain::text;

use crate::library::CatalogStore;
use crate::port::assistant::{FieldAnswer, QueryAnswer, QueryRequest, QueryTranslator, TranslatedQuery};
use crate::port::system::Clock;
use crate::search::{QueryTranslation, TranslateOutcome};

/// Turns a sentence in plain words into the library's filters with the AI. Only names the library knows become
/// filters; every other name is reported as not understood, never guessed. It never searches by itself.
pub struct TranslateQuery {
    store: Arc<CatalogStore>,
    translator: Arc<dyn QueryTranslator>,
    clock: Arc<dyn Clock>,
}

impl TranslateQuery {
    pub fn new(store: Arc<CatalogStore>, translator: Arc<dyn QueryTranslator>, clock: Arc<dyn Clock>) -> Self {
        Self { store, translator, clock }
    }

    pub fn run(&self, sentence: &str) -> TranslateOutcome {
        let Some(library) = self.store.current() else {
            return TranslateOutcome::LibraryNotOpen;
        };
        let sentence = text::trim(sentence);
        if sentence.is_empty() {
            return TranslateOutcome::BlankSentence;
        }
        let catalog = &library.catalog;
        let request = QueryRequest {
            sentence: sentence.to_owned(),
            today: self.clock.today(),
            categories: catalog.categories().values().cloned().collect(),
            tags: catalog.tags().iter().cloned().collect(),
            correspondents: correspondents_of(catalog),
            fields: catalog.settings().field_definitions.clone(),
            tax_areas: TaxArea::ALL.to_vec(),
        };
        match self.translator.translate(&request) {
            QueryAnswer::Failed { reason, detail } => TranslateOutcome::Failed { reason, detail },
            QueryAnswer::Translated { query, model } => {
                TranslateOutcome::Translated(translation(&query, catalog, model))
            }
        }
    }
}

/// The answer checked against `catalog`: what it knows becomes a filter, the rest is not understood.
fn translation(answer: &TranslatedQuery, catalog: &Catalog, model: String) -> QueryTranslation {
    let mut not_understood = Vec::new();
    let missing = &mut not_understood;
    let category = answer.category.as_deref().and_then(|name| known(category_named(name, catalog), name, missing));
    let tags = answer.tags.iter().filter_map(|name| known(tag_named(name, catalog), name, missing)).collect();
    let correspondent =
        answer.correspondent.as_deref().and_then(|name| known(correspondent_named(name, catalog), name, missing));
    let fields =
        answer.fields.iter().filter_map(|field| known(condition_of(field, catalog), &field.key, missing)).collect();
    let dates = match DateRange::new(answer.date_from, answer.date_to) {
        Ok(range) => (range.from.is_some() || range.to.is_some()).then_some(range),
        Err(_) => known(None, "dates", missing),
    };
    let text = answer.text.as_deref().map(text::trim).filter(|text| !text.is_empty()).map(str::to_owned);
    let query = DocumentQuery {
        text,
        category,
        dates,
        correspondent,
        fields,
        tags: dms_domain::query::TagFilter { all: tags, ..Default::default() },
        ..DocumentQuery::default()
    };
    QueryTranslation {
        query,
        payment: answer.payment,
        tax_year: answer.tax_year,
        tax_area: answer.tax_area,
        not_understood,
        model,
    }
}

/// `found`, or `None` with `raw` noted as not understood.
fn known<T>(found: Option<T>, raw: &str, not_understood: &mut Vec<String>) -> Option<T> {
    if found.is_none() {
        not_understood.push(text::trim(raw).to_owned());
    }
    found
}

/// An existing category by its name or its path ("Home / Energy"), ignoring case.
fn category_named(name: &str, catalog: &Catalog) -> Option<CategoryId> {
    let wanted = text::lowercase(text::trim(name));
    let categories = catalog.categories();
    categories
        .values()
        .find(|category| {
            let mut names: Vec<&str> = std::iter::successors(Some(*category), |current| {
                current.parent_id.as_ref().and_then(|parent| categories.get(parent))
            })
            .take(categories.len())
            .map(|category| category.name.as_str())
            .collect();
            names.reverse();
            text::lowercase(&category.name) == wanted || text::lowercase(&names.join(" / ")) == wanted
        })
        .map(|category| category.id.clone())
}

fn tag_named(name: &str, catalog: &Catalog) -> Option<TagName> {
    let tag = TagName::parse(name)?;
    catalog.tags().get(&tag).cloned()
}

/// The library's spelling of a correspondent with exactly this name, ignoring case.
fn correspondent_named(name: &str, catalog: &Catalog) -> Option<String> {
    let wanted = text::lowercase(text::trim(name));
    correspondents_of(catalog).into_iter().find(|known| text::lowercase(known) == wanted)
}

/// A condition on a field the library defines, by key or label, with a value of the field's type.
fn condition_of(answer: &FieldAnswer, catalog: &Catalog) -> Option<FieldCondition> {
    let settings = catalog.settings();
    let wanted = text::lowercase(text::trim(&answer.key));
    let definition = settings.field_definitions.iter().find(|definition| {
        text::lowercase(definition.key.value()) == wanted || text::lowercase(&definition.label) == wanted
    })?;
    let value = match answer.value.as_deref() {
        None => None,
        Some(raw) => Some(FieldValues::parse(definition.field_type, raw, settings.layout.default_currency.as_ref())?),
    };
    FieldCondition::new(definition.key.clone(), answer.comparison, value).ok()
}

/// The trimmed correspondents of the catalog, one spelling per name ignoring case.
fn correspondents_of(catalog: &Catalog) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    catalog
        .documents()
        .values()
        .filter_map(|document| document.metadata.correspondent.as_deref().map(text::trim))
        .filter(|name| !name.is_empty() && seen.insert(text::lowercase(name)))
        .map(str::to_owned)
        .collect()
}
