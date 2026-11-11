//! The facet filters as tokens of the search field, a saved view's query as tokens that stay, and the tokens the
//! typed text suggests (Kotlin `FilterTokens`). Tokens and facet chips render the same filters.

use std::collections::HashSet;

use chrono::{Datelike, SecondsFormat};
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentKind};
use dms_domain::iso_date;
use dms_domain::query::{DateRange, DocumentQuery};
use dms_domain::taxonomy::{CategoryId, TagName};
use dms_domain::text;

use crate::features::library::model::{Facet, FacetFilter, YearFilter};

const MIN_TAG_LETTERS: usize = 2;
const MIN_CORRESPONDENT_LETTERS: usize = 3;
const MAX_SUGGESTIONS: usize = 3;
const CATEGORY_SEPARATOR: &str = " › ";

/// One condition in the search field, `key: value`; one of a saved view's own conditions has no filter and stays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterToken {
    pub key: String,
    pub value: String,
    pub filter: Option<FacetFilter>,
}

impl FilterToken {
    fn fixed(key: &str, value: impl Into<String>) -> Self {
        Self { key: key.to_owned(), value: value.into(), filter: None }
    }
}

pub fn facet_of(filter: &FacetFilter) -> Facet {
    match filter {
        FacetFilter::Category(_) => Facet::Category,
        FacetFilter::Tag(_) => Facet::Tag,
        FacetFilter::Year(_) => Facet::Year,
        FacetFilter::Correspondent(_) => Facet::Correspondent,
        FacetFilter::Type(_) => Facet::Type,
        FacetFilter::Folder(_) => Facet::Folder,
    }
}

pub fn token_of(filter: &FacetFilter, catalog: &Catalog) -> FilterToken {
    FilterToken {
        key: facet_of(filter).key().to_owned(),
        value: value_of(filter, catalog),
        filter: Some(filter.clone()),
    }
}

/// What a facet chip shows while `filter` is active, such as `Home › Energy` or `PDF`.
pub fn value_of(filter: &FacetFilter, catalog: &Catalog) -> String {
    match filter {
        FacetFilter::Category(Some(id)) => category_path(id, catalog),
        FacetFilter::Category(None) => "Without a category".to_owned(),
        FacetFilter::Tag(tag) => tag.value().to_owned(),
        FacetFilter::Year(year) => year.label(),
        FacetFilter::Correspondent(name) => name.clone(),
        FacetFilter::Type(kind) => kind_label(*kind).to_owned(),
        FacetFilter::Folder(path) => path.value().to_owned(),
    }
}

/// The name of a kind of file as every screen writes it (Kotlin `KindStyles.label`).
pub fn kind_label(kind: DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Pdf => "PDF",
        DocumentKind::Image => "Image",
        DocumentKind::Office => "Office",
        DocumentKind::Other => "Other",
    }
}

/// The category with its parents, as in `Home › Energy`.
pub fn category_path(id: &CategoryId, catalog: &Catalog) -> String {
    let mut names = Vec::new();
    let mut visited = HashSet::new();
    let mut current = catalog.categories().get(id);
    while let Some(category) = current {
        if !visited.insert(&category.id) {
            break;
        }
        names.push(category.name.as_str());
        current = category.parent_id.as_ref().and_then(|parent| catalog.categories().get(parent));
    }
    if names.is_empty() {
        return "Unknown category".to_owned();
    }
    names.reverse();
    names.join(CATEGORY_SEPARATOR)
}

/// The conditions a saved view keeps in its query, as tokens without a way to remove them.
pub fn saved_tokens(query: &DocumentQuery, catalog: &Catalog) -> Vec<FilterToken> {
    let non_blank = |value: &Option<String>| value.clone().filter(|value| !text::is_blank(value));
    let mut tokens: Vec<FilterToken> = [
        non_blank(&query.text).map(|value| FilterToken::fixed("text", value)),
        non_blank(&query.content_text).map(|value| FilterToken::fixed("text", value)),
        query.category.as_ref().map(|id| FilterToken::fixed("category", category_path(id, catalog))),
        query.uncategorized.then(|| FilterToken::fixed("category", "Without a category")),
        query.untagged.then(|| FilterToken::fixed("tag", "none")),
        (!query.tags.any.is_empty())
            .then(|| FilterToken::fixed("tag", format!("any of {}", sorted_tags(&query.tags.any)))),
        query.dates.map(date_token),
        query.correspondent.as_ref().map(|name| FilterToken::fixed("from", name.clone())),
        query
            .folder
            .as_ref()
            .filter(|folder| !folder.is_root())
            .map(|folder| FilterToken::fixed("folder", folder.value())),
        query.added_since.map(|since| {
            FilterToken::fixed("added", format!("since {}", since.to_rfc3339_opts(SecondsFormat::AutoSi, true)))
        }),
    ]
    .into_iter()
    .flatten()
    .collect();
    tokens.extend(sorted(&query.tags.all).into_iter().map(|tag| FilterToken::fixed("tag", tag.value())));
    tokens.extend(sorted(&query.tags.none).into_iter().map(|tag| FilterToken::fixed("not tag", tag.value())));
    tokens.extend(query.fields.iter().map(|condition| {
        let label = catalog
            .settings()
            .field(&condition.key)
            .map_or_else(|| condition.key.value().to_owned(), |definition| definition.label.clone());
        FilterToken::fixed("field", label)
    }));
    tokens
}

/// The tokens the typed text suggests: a known tag, a four-digit year or a correspondent of `documents`, at most
/// three, without the filters already `active`.
pub fn suggestions(
    typed: &str,
    documents: &[&Document],
    active: &[FacetFilter],
    catalog: &Catalog,
) -> Vec<FilterToken> {
    let wanted = text::lowercase(text::trim(typed));
    if wanted.is_empty() {
        return Vec::new();
    }
    let letters = text::utf16_len(&wanted);
    let tags = sorted(catalog.tags())
        .into_iter()
        .filter(|tag| letters >= MIN_TAG_LETTERS && text::lowercase(tag.value()).contains(&wanted))
        .map(|tag| FacetFilter::Tag(tag.clone()));
    let year = (wanted.len() == 4 && wanted.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| wanted.parse().ok())
        .flatten()
        .map(|year| FacetFilter::Year(YearFilter { year, and_earlier: false }));
    let correspondents = distinct_correspondents(documents)
        .into_iter()
        .filter(|name| letters >= MIN_CORRESPONDENT_LETTERS && text::lowercase(name).contains(&wanted))
        .map(FacetFilter::Correspondent);
    tags.chain(year)
        .chain(correspondents)
        .filter(|filter| !active.contains(filter))
        .take(MAX_SUGGESTIONS)
        .map(|filter| token_of(&filter, catalog))
        .collect()
}

/// The trimmed correspondents, one spelling per name ignoring case (the first seen), sorted ignoring case.
fn distinct_correspondents(documents: &[&Document]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut names: Vec<(String, String)> = documents
        .iter()
        .filter_map(|document| document.metadata.correspondent.as_deref().map(text::trim))
        .filter(|name| seen.insert(text::lowercase(name)))
        .map(|name| (text::lowercase(name), name.to_owned()))
        .collect();
    names.sort_by(|first, second| text::compare(&first.0, &second.0));
    names.into_iter().map(|(_, name)| name).collect()
}

fn sorted<'a>(tags: impl IntoIterator<Item = &'a TagName>) -> Vec<&'a TagName> {
    let mut sorted: Vec<&TagName> = tags.into_iter().collect();
    sorted.sort();
    sorted
}

fn sorted_tags<'a>(tags: impl IntoIterator<Item = &'a TagName>) -> String {
    sorted(tags).iter().map(|tag| tag.value()).collect::<Vec<_>>().join(", ")
}

fn date_token(range: DateRange) -> FilterToken {
    if let Some(year) = year_of(range) {
        return FilterToken::fixed("year", year.label());
    }
    let end = |date: Option<chrono::NaiveDate>| date.map_or_else(|| "…".to_owned(), iso_date);
    FilterToken::fixed("date", format!("{} – {}", end(range.from), end(range.to)))
}

/// The year filter `range` stands for, if it covers exactly one year or ends with one.
fn year_of(range: DateRange) -> Option<YearFilter> {
    let to = range.to?;
    if to.month() != 12 || to.day() != 31 {
        return None;
    }
    match range.from {
        None => Some(YearFilter { year: to.year(), and_earlier: true }),
        Some(_) if DateRange::year(to.year()).ok() == Some(range) => {
            Some(YearFilter { year: to.year(), and_earlier: false })
        }
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, category_id, date, path, tag};

    #[test]
    fn category_values_join_the_parents() {
        let catalog =
            TestCatalog::new().category("c-home", "Home", None).category("c-energy", "Energy", Some("c-home")).build();
        assert_eq!(value_of(&FacetFilter::Category(Some(category_id("c-energy"))), &catalog), "Home › Energy");
        assert_eq!(value_of(&FacetFilter::Category(Some(category_id("gone"))), &catalog), "Unknown category");
        assert_eq!(value_of(&FacetFilter::Category(None), &catalog), "Without a category");
        assert_eq!(value_of(&FacetFilter::Year(YearFilter { year: 2017, and_earlier: true }), &catalog), "Until 2017");
    }

    #[test]
    fn saved_tokens_list_every_condition_in_order() {
        let catalog = TestCatalog::new().build();
        let mut query = DocumentQuery {
            text: Some("bill".to_owned()),
            untagged: true,
            dates: Some(DateRange::new(Some(date(2025, 1, 1)), None).expect("range")),
            correspondent: Some("Sample Utility".to_owned()),
            folder: Some(path("Home")),
            ..DocumentQuery::default()
        };
        query.tags.any.insert(tag("b"));
        query.tags.any.insert(tag("a"));
        query.tags.all.insert(tag("x"));
        query.tags.none.insert(tag("y"));
        let tokens: Vec<String> =
            saved_tokens(&query, &catalog).iter().map(|token| format!("{}: {}", token.key, token.value)).collect();
        assert_eq!(
            tokens,
            [
                "text: bill",
                "tag: none",
                "tag: any of a, b",
                "date: 2025-01-01 – …",
                "from: Sample Utility",
                "folder: Home",
                "tag: x",
                "not tag: y"
            ]
        );
    }

    #[test]
    fn year_ranges_read_as_years() {
        assert_eq!(date_token(DateRange::year(2024).expect("year")).value, "2024");
        assert_eq!(date_token(DateRange::until_end_of(2017).expect("year")).value, "Until 2017");
        assert_eq!(date_token(DateRange::year(2024).expect("year")).key, "year");
    }

    #[test]
    fn suggestions_offer_tags_years_and_correspondents_without_active_filters() {
        let catalog = TestCatalog::new()
            .document("a.pdf", |document| {
                document.tags(&["invoice", "involved"]).correspondent(" Invoicing Ltd ");
            })
            .document("b.pdf", |document| {
                document.correspondent("invoicing ltd");
            })
            .build();
        let documents: Vec<&Document> = catalog.documents().values().collect();
        let active = [FacetFilter::Tag(tag("involved"))];
        let values: Vec<String> = suggestions("invo", &documents, &active, &catalog)
            .iter()
            .map(|token| format!("{}: {}", token.key, token.value))
            .collect();
        assert_eq!(values, ["tag: invoice", "from: Invoicing Ltd"]);
        let year: Vec<String> =
            suggestions("2025", &documents, &[], &catalog).iter().map(|token| token.value.clone()).collect();
        assert_eq!(year, ["2025"]);
        assert_eq!(suggestions("i", &documents, &[], &catalog).len(), 0);
    }
}
