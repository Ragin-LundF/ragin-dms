//! Derives the library screen from the catalog and the user's choices (Kotlin `LibraryStates`). The facet counts
//! and the suggestions come from the documents of the sidebar's view before the facet filters and the search text
//! narrow them.

use std::collections::HashSet;

use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::field::FieldValues;
use dms_domain::iso_date;
use dms_domain::query::{DocumentQuery, QueryContext};
use dms_domain::text;

use super::facets::{self, FacetOption};
use super::filter::{self, Matches};
use super::rows::{self, Row};
use super::tokens::{self, FilterToken};
use super::trees;
use crate::features::library::dto::params::LibraryFilterDto;
use crate::features::library::dto::state::{
    CategoryTileDto, LibraryCategoryDto, LibraryCountsDto, LibraryFacetDto, LibraryFacetOptionDto,
    LibraryFacetOptionsDto, LibraryGroupDto, LibraryRowDto, LibrarySavedViewDto, LibraryStateDto, LibraryTokenDto,
};
use crate::features::library::model::{Facet, FacetFilter, Moment, View, ViewParams};

pub fn present_library(
    catalog: &Catalog,
    params: &ViewParams,
    matches: &Matches<'_>,
    moment: &Moment,
) -> LibraryStateDto {
    let in_view = filter::documents(catalog, &params.view, &[], "", moment, &Matches { text: None, ..*matches });
    let visible = filter::documents(catalog, &params.view, &params.filters, &params.query, moment, matches);
    let filtered = !params.filters.is_empty() || !text::is_blank(&params.query);
    let rows = rows::sorted(visible.iter().map(|document| rows::row_of(document, catalog)).collect(), params.sort);
    let order = rows.iter().map(|row| row.document.id.value().to_owned()).collect();
    let all_documents = params.view == View::Folder(dms_domain::library::RelativePath::root());
    let no_reviews = HashSet::new();
    let counts = rows::counts(catalog, matches.review.unwrap_or(&no_reviews), moment);
    LibraryStateDto {
        title: filter::title_of(&params.view, catalog),
        counts: LibraryCountsDto {
            all: count(counts.all),
            inbox: count(counts.inbox),
            needs_review: count(counts.needs_review),
            added_this_month: count(counts.added_this_month),
            folders: count(counts.folders),
        },
        categories: categories(catalog),
        tokens: saved_tokens(params, catalog)
            .into_iter()
            .chain(params.filters.iter().map(|filter| tokens::token_of(filter, catalog)))
            .map(token_dto)
            .collect(),
        suggestions: tokens::suggestions(&params.query, &in_view, &params.filters, catalog)
            .into_iter()
            .map(token_dto)
            .collect(),
        facets: facets::of(&in_view, catalog, moment)
            .into_iter()
            .map(|(facet, options)| LibraryFacetOptionsDto {
                facet: facet_dto(facet),
                options: options.into_iter().map(option_dto).collect(),
            })
            .collect(),
        tiles: if all_documents && !filtered { tiles(catalog) } else { Vec::new() },
        order,
        groups: rows::groups(rows, params.group_by, moment, catalog)
            .into_iter()
            .map(|group| LibraryGroupDto { title: group.title, rows: group.rows.iter().map(row_dto).collect() })
            .collect(),
        smart_folders: smart_folders(catalog, moment),
        query_problem: query_problem(params).map(str::to_owned),
    }
}

/// What the library shows now as one query: view, filters and the trimmed search text, as content term while the
/// content search is on. `None` where the view cannot be a query (see [`query_problem`]).
pub fn current_query(catalog: &Catalog, params: &ViewParams, moment: &Moment) -> Option<DocumentQuery> {
    if query_problem(params).is_some() {
        return None;
    }
    let base = filter::query_of(&params.view, &params.filters, moment, catalog).unwrap_or_default();
    let typed = Some(text::trim(&params.query).to_owned()).filter(|typed| !typed.is_empty());
    Some(if params.content_search {
        DocumentQuery { content_text: typed, ..base }
    } else {
        DocumentQuery { text: typed, ..base }
    })
}

/// Why the view cannot become a saved view or an export, which both take it as one query.
pub fn query_problem(params: &ViewParams) -> Option<&'static str> {
    if params.filters.iter().any(|filter| matches!(filter, FacetFilter::Type(_))) {
        return Some("A saved view or an export cannot keep the type filter yet");
    }
    (params.view == View::NeedsReview).then_some("Needs review cannot be saved or exported as a view")
}

pub fn row_dto(row: &Row<'_>) -> LibraryRowDto {
    let document = row.document;
    let mut tags: Vec<_> = document.tags.iter().collect();
    tags.sort();
    LibraryRowDto {
        id: document.id.value().to_owned(),
        path: document.path.value().to_owned(),
        file_name: document.path.file_name().to_owned(),
        folder: document.path.parent().filter(|parent| !parent.is_root()).map(|parent| parent.value().to_owned()),
        title: document.metadata.title.clone(),
        kind: document.kind().into(),
        category_name: row.category_name.map(str::to_owned),
        correspondent: document.metadata.correspondent.clone(),
        document_date: document.metadata.document_date.map(iso_date),
        tags: tags.into_iter().map(|tag| tag.value().to_owned()).collect(),
        amount: row.amount.map(FieldValues::format),
        due_date: row.due_date.map(iso_date),
    }
}

/// The smart folders with their counts, in the order the user created them.
fn smart_folders(catalog: &Catalog, moment: &Moment) -> Vec<LibrarySavedViewDto> {
    let context = QueryContext { catalog, today: moment.today(), zone: moment.zone };
    catalog
        .settings()
        .saved_searches
        .iter()
        .map(|search| LibrarySavedViewDto {
            id: search.id.value().to_owned(),
            name: search.name.clone(),
            count: (!search.query.needs_content()).then(|| count(search.query.filter(&context).len())),
            invalid: search.query.refers_to_missing(catalog),
        })
        .collect()
}

fn saved_tokens(params: &ViewParams, catalog: &Catalog) -> Vec<FilterToken> {
    let View::SmartFolder(id) = &params.view else {
        return Vec::new();
    };
    catalog
        .settings()
        .saved_searches
        .iter()
        .find(|search| &search.id == id)
        .map(|search| tokens::saved_tokens(&search.query, catalog))
        .unwrap_or_default()
}

fn categories(catalog: &Catalog) -> Vec<LibraryCategoryDto> {
    trees::categories(catalog.categories().values(), catalog.documents().values())
        .into_iter()
        .map(|entry| LibraryCategoryDto {
            id: entry.category.id.value().to_owned(),
            name: entry.category.name.clone(),
            parent_id: entry.category.parent_id.as_ref().map(|id| id.value().to_owned()),
            depth: count(entry.depth),
            document_count: count(entry.document_count),
        })
        .collect()
}

fn tiles(catalog: &Catalog) -> Vec<CategoryTileDto> {
    rows::tiles(catalog)
        .into_iter()
        .map(|tile| CategoryTileDto {
            category_id: tile.category_id.value().to_owned(),
            name: tile.name.to_owned(),
            count: count(tile.count),
            kinds: tile.kinds.into_iter().map(Into::into).collect(),
        })
        .collect()
}

fn token_dto(token: FilterToken) -> LibraryTokenDto {
    LibraryTokenDto { key: token.key, value: token.value, filter: token.filter.as_ref().map(LibraryFilterDto::of) }
}

fn option_dto(option: FacetOption) -> LibraryFacetOptionDto {
    LibraryFacetOptionDto {
        filter: LibraryFilterDto::of(&option.filter),
        label: option.label,
        count: count(option.count),
        depth: count(option.depth),
        missing: option.missing,
    }
}

fn facet_dto(facet: Facet) -> LibraryFacetDto {
    match facet {
        Facet::Category => LibraryFacetDto::Category,
        Facet::Tag => LibraryFacetDto::Tag,
        Facet::Year => LibraryFacetDto::Year,
        Facet::Correspondent => LibraryFacetDto::Correspondent,
        Facet::Type => LibraryFacetDto::Type,
        Facet::Folder => LibraryFacetDto::Folder,
    }
}

/// A count for the UI; a library never holds more than `u32::MAX` of anything.
pub fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// The documents of `catalog` as rows, for the folder view.
pub fn rows_of<'a>(documents: &[&'a Document], catalog: &'a Catalog) -> Vec<LibraryRowDto> {
    documents.iter().map(|document| row_dto(&rows::row_of(document, catalog))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::model::{GroupKey, YearFilter};
    use crate::features::library::test_catalog::{TestCatalog, category_id, moment, path};
    use dms_domain::document::DocumentKind;

    fn params(view: View, filters: Vec<FacetFilter>, query: &str) -> ViewParams {
        ViewParams {
            view,
            filters,
            query: query.to_owned(),
            sort: None,
            group_by: GroupKey::Month,
            content_search: true,
        }
    }

    fn sample() -> Catalog {
        TestCatalog::new()
            .category("c-home", "Home", None)
            .document("Home/a.pdf", |document| {
                document.category("c-home").title("Rent");
            })
            .document("b.pdf", |_| {})
            .build()
    }

    #[test]
    fn facet_counts_come_from_the_view_before_filters_and_tiles_only_show_unfiltered() {
        let catalog = sample();
        let unfiltered =
            present_library(&catalog, &params(View::Folder(path("")), vec![], ""), &Matches::default(), &moment());
        assert_eq!(unfiltered.tiles.len(), 1);
        assert_eq!(unfiltered.order.len(), 2);
        let filtered = params(View::Folder(path("")), vec![FacetFilter::Category(Some(category_id("c-home")))], "");
        let state = present_library(&catalog, &filtered, &Matches::default(), &moment());
        assert!(state.tiles.is_empty());
        assert_eq!(state.order.len(), 1);
        let categories = &state.facets[0].options;
        assert_eq!(categories.iter().map(|option| option.count).collect::<Vec<_>>(), [1, 1]);
        assert_eq!(state.tokens.len(), 1);
        assert_eq!(state.tokens[0].value, "Home");
    }

    #[test]
    fn the_type_filter_and_needs_review_cannot_become_a_query() {
        let typed = params(View::Folder(path("")), vec![FacetFilter::Type(DocumentKind::Pdf)], "");
        assert_eq!(query_problem(&typed), Some("A saved view or an export cannot keep the type filter yet"));
        assert_eq!(
            query_problem(&params(View::NeedsReview, vec![], "")),
            Some("Needs review cannot be saved or exported as a view")
        );
        assert!(current_query(&sample(), &typed, &moment()).is_none());
    }

    #[test]
    fn the_current_query_keeps_the_search_as_content_term_while_content_search_is_on() {
        let catalog = sample();
        let mut chosen = params(
            View::Folder(path("")),
            vec![FacetFilter::Year(YearFilter { year: 2024, and_earlier: false })],
            "  rent ",
        );
        let query = current_query(&catalog, &chosen, &moment()).expect("query");
        assert_eq!(query.content_text.as_deref(), Some("rent"));
        assert!(query.text.is_none() && query.dates.is_some());
        chosen.content_search = false;
        assert_eq!(current_query(&catalog, &chosen, &moment()).expect("query").text.as_deref(), Some("rent"));
    }

    #[test]
    fn rows_show_the_folder_and_formatted_values() {
        let catalog = sample();
        let state =
            present_library(&catalog, &params(View::Folder(path("")), vec![], ""), &Matches::default(), &moment());
        let rows: Vec<&LibraryRowDto> = state.groups.iter().flat_map(|group| &group.rows).collect();
        let rent = rows.iter().find(|row| row.title.as_deref() == Some("Rent")).expect("rent");
        assert_eq!(rent.folder.as_deref(), Some("Home"));
        assert_eq!(rent.category_name.as_deref(), Some("Home"));
        let top = rows.iter().find(|row| row.file_name == "b.pdf").expect("top");
        assert_eq!(top.folder, None);
    }
}
