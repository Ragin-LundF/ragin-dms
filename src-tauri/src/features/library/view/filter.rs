//! Which documents the library shows (Kotlin `LibraryFilter`): the sidebar's view, the facet filters and the search
//! text together. The one place that decides it, so the list, the counts, saving a view and exporting agree.

use std::collections::HashSet;

use dms_domain::bill::BillStatus;
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId};
use dms_domain::query::{DateRange, DocumentQuery, QueryContext};
use dms_domain::tax::TaxFilter;
use dms_domain::text;

use crate::features::library::model::{FacetFilter, Moment, View, YearFilter};

/// What the search found beyond the catalog: documents whose text matches the search (`text`), the matches of a
/// saved view that needs the documents' text (`query`, `None` when it needs none), and the documents with an AI
/// suggestion waiting for review.
#[derive(Debug, Default, Clone, Copy)]
pub struct Matches<'m> {
    pub text: Option<&'m HashSet<DocumentId>>,
    pub query: Option<&'m HashSet<DocumentId>>,
    pub review: Option<&'m HashSet<DocumentId>>,
}

/// The documents `view` shows with `filters` and the search text `query`; "Added this month" newest first, all
/// others by path ignoring case.
pub fn documents<'a>(
    catalog: &'a Catalog,
    view: &View,
    filters: &[FacetFilter],
    query: &str,
    moment: &Moment,
    matches: &Matches<'_>,
) -> Vec<&'a Document> {
    let context = QueryContext { catalog, today: moment.today(), zone: moment.zone };
    let Some(selected) = query_of(view, filters, moment, catalog) else {
        return Vec::new();
    };
    let kind = filters.iter().find_map(|filter| match filter {
        FacetFilter::Type(kind) => Some(*kind),
        _ => None,
    });
    let payment = filters.iter().find_map(|filter| match filter {
        FacetFilter::Payment(status) => Some(*status),
        _ => None,
    });
    let typed = DocumentQuery { text: Some(text::lowercase(text::trim(query))), ..DocumentQuery::default() };
    let contains = |ids: Option<&HashSet<DocumentId>>, id: &DocumentId| ids.is_some_and(|ids| ids.contains(id));
    let shown = catalog.documents().values().filter(|document| {
        selected.matches(document, &context)
            && kind.is_none_or(|kind| document.kind() == kind)
            && payment.is_none_or(|status| BillStatus::of(document, catalog.settings(), moment.today()) == Some(status))
            && (*view != View::NeedsReview || contains(matches.review, &document.id))
            && matches.query.is_none_or(|ids| ids.contains(&document.id))
            && (typed.matches(document, &context) || contains(matches.text, &document.id))
    });
    if *view == View::Recent {
        let mut newest: Vec<&Document> = shown.collect();
        newest.sort_by_key(|document| std::cmp::Reverse(document.added_at));
        return newest;
    }
    let mut keyed: Vec<(String, &Document)> =
        shown.map(|document| (text::lowercase(document.path.value()), document)).collect();
    keyed.sort_by(|first, second| text::compare(&first.0, &second.0));
    keyed.into_iter().map(|(_, document)| document).collect()
}

/// The year a document belongs to: of its document date, or of when it was added.
pub fn year_of(document: &Document, moment: &Moment) -> i32 {
    use chrono::Datelike;
    document.metadata.document_date.map_or_else(|| moment.year_of(document.added_at), |date| date.year())
}

pub fn title_of(view: &View, catalog: &Catalog) -> String {
    match view {
        View::Folder(path) if path.is_root() => "All documents".to_owned(),
        View::Folder(path) => path.file_name().to_owned(),
        View::Inbox => "Inbox".to_owned(),
        View::NeedsReview => "Needs review".to_owned(),
        View::Recent => "Added this month".to_owned(),
        View::Folders => "Folders".to_owned(),
        View::SmartFolder(id) => catalog
            .settings()
            .saved_searches
            .iter()
            .find(|search| &search.id == id)
            .map_or_else(|| "Saved view".to_owned(), |search| search.name.clone()),
    }
}

/// The view with its filters as one query, shared by the library, saved views and exports. `None` where the view
/// shows no documents: a saved view the catalog no longer has, the inbox while it is off, and the folder view,
/// which lists its documents itself. The type and payment filters and "Needs review" cannot be part of a query; [`documents`]
/// applies them on top.
pub fn query_of(view: &View, filters: &[FacetFilter], moment: &Moment, catalog: &Catalog) -> Option<DocumentQuery> {
    let base = base_query_of(view, moment, catalog)?;
    Some(filters.iter().fold(base, narrowed_by))
}

fn base_query_of(view: &View, moment: &Moment, catalog: &Catalog) -> Option<DocumentQuery> {
    let query = DocumentQuery::default();
    match view {
        View::Folder(path) => Some(DocumentQuery { folder: Some(path.clone()), ..query }),
        View::Inbox => {
            let inbox = &catalog.settings().layout.inbox;
            inbox.enabled.then(|| DocumentQuery { folder: Some(inbox.path.clone()), ..query })
        }
        View::NeedsReview => Some(query),
        View::Recent => Some(DocumentQuery { added_since: Some(moment.start_of_month()), ..query }),
        View::Folders => None,
        View::SmartFolder(id) => {
            catalog.settings().saved_searches.iter().find(|search| &search.id == id).map(|search| search.query.clone())
        }
    }
}

fn narrowed_by(mut query: DocumentQuery, filter: &FacetFilter) -> DocumentQuery {
    match filter {
        FacetFilter::Category(id) => {
            query.uncategorized = id.is_none();
            query.category.clone_from(id);
        }
        FacetFilter::Tag(tag) => {
            query.tags.all.insert(tag.clone());
        }
        FacetFilter::Year(year) => query.dates = Some(dates_of(*year)),
        FacetFilter::Correspondent(name) => query.correspondent = Some(name.clone()),
        FacetFilter::Type(_) | FacetFilter::Payment(_) => {}
        FacetFilter::Folder(path) => query.folder = Some(path.clone()),
        FacetFilter::Tax(tax) => {
            query.tax = Some(TaxFilter { year: Some(tax.year), area: tax.area, tag_fallback: None });
        }
        FacetFilter::Dates(range) => query.dates = Some(*range),
        FacetFilter::Field(condition) => query.fields.push(condition.clone()),
    }
    query
}

/// # Panics
/// Never for a [`YearFilter`] parsed by the commands, which check the year fits a calendar.
pub fn dates_of(year: YearFilter) -> DateRange {
    let range = if year.and_earlier { DateRange::until_end_of(year.year) } else { DateRange::year(year.year) };
    range.expect("invariant: the year was checked when the filter was parsed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, category_id, date, moment, path, tag};
    use dms_domain::query::{SavedSearch, SavedSearchId};

    fn paths(documents: &[&Document]) -> Vec<String> {
        documents.iter().map(|document| document.path.value().to_owned()).collect()
    }

    fn all() -> View {
        View::Folder(path(""))
    }

    #[test]
    fn all_documents_are_sorted_by_path_ignoring_case() {
        let catalog =
            TestCatalog::new().document("b.pdf", |_| {}).document("A.pdf", |_| {}).document("c.pdf", |_| {}).build();
        let shown = documents(&catalog, &all(), &[], "", &moment(), &Matches::default());
        assert_eq!(paths(&shown), ["A.pdf", "b.pdf", "c.pdf"]);
    }

    #[test]
    fn added_this_month_starts_at_the_first_day_of_the_month_and_lists_newest_first() {
        let catalog = TestCatalog::new()
            .document("old.pdf", |document| {
                document.added("2025-09-30T21:59:59Z");
            })
            .document("first.pdf", |document| {
                // Berlin is UTC+2 in October: the month starts at 2025-09-30T22:00Z.
                document.added("2025-09-30T22:00:00Z");
            })
            .document("new.pdf", |document| {
                document.added("2025-10-10T08:00:00Z");
            })
            .build();
        let shown = documents(&catalog, &View::Recent, &[], "", &moment(), &Matches::default());
        assert_eq!(paths(&shown), ["new.pdf", "first.pdf"]);
    }

    #[test]
    fn a_category_filter_includes_subcategories_and_none_means_without_a_category() {
        let catalog = TestCatalog::new()
            .category("c-home", "Home", None)
            .category("c-energy", "Energy", Some("c-home"))
            .document("a.pdf", |document| {
                document.category("c-energy");
            })
            .document("b.pdf", |_| {})
            .build();
        let home = [FacetFilter::Category(Some(category_id("c-home")))];
        assert_eq!(paths(&documents(&catalog, &all(), &home, "", &moment(), &Matches::default())), ["a.pdf"]);
        let without = [FacetFilter::Category(None)];
        assert_eq!(paths(&documents(&catalog, &all(), &without, "", &moment(), &Matches::default())), ["b.pdf"]);
    }

    #[test]
    fn the_search_text_matches_names_and_tags_or_the_content_matches() {
        let catalog = TestCatalog::new()
            .document("invoice.pdf", |_| {})
            .document("other.pdf", |document| {
                document.tags(&["Invoice"]);
            })
            .document("scan.pdf", |_| {})
            .build();
        let scan = catalog.find_by_path(&path("scan.pdf")).expect("scan").id.clone();
        let content = HashSet::from([scan]);
        let matches = Matches { text: Some(&content), ..Matches::default() };
        let shown = documents(&catalog, &all(), &[], " INVOICE ", &moment(), &matches);
        assert_eq!(paths(&shown), ["invoice.pdf", "other.pdf", "scan.pdf"]);
    }

    #[test]
    fn needs_review_shows_only_the_documents_waiting_for_review() {
        let catalog = TestCatalog::new().document("a.pdf", |_| {}).document("b.pdf", |_| {}).build();
        let b = catalog.find_by_path(&path("b.pdf")).expect("b").id.clone();
        let review = HashSet::from([b]);
        let matches = Matches { review: Some(&review), ..Matches::default() };
        assert_eq!(paths(&documents(&catalog, &View::NeedsReview, &[], "", &moment(), &matches)), ["b.pdf"]);
    }

    #[test]
    fn the_inbox_is_empty_while_it_is_off() {
        let catalog = TestCatalog::new().document("Inbox/a.pdf", |_| {}).build();
        assert_eq!(documents(&catalog, &View::Inbox, &[], "", &moment(), &Matches::default()).len(), 0);
        let on = TestCatalog::new()
            .document("Inbox/a.pdf", |_| {})
            .settings(|settings| settings.layout.inbox.enabled = true)
            .build();
        assert_eq!(paths(&documents(&on, &View::Inbox, &[], "", &moment(), &Matches::default())), ["Inbox/a.pdf"]);
    }

    #[test]
    fn a_facet_tag_adds_to_the_saved_view_tags_while_a_facet_folder_replaces_its_folder() {
        let mut saved = DocumentQuery { folder: Some(path("a")), ..DocumentQuery::default() };
        saved.tags.all.insert(tag("x"));
        let catalog = TestCatalog::new()
            .document("a/one.pdf", |document| {
                document.tags(&["x", "y"]);
            })
            .document("b/two.pdf", |document| {
                document.tags(&["x", "y"]);
            })
            .document("b/three.pdf", |document| {
                document.tags(&["x"]);
            })
            .settings(|settings| {
                settings.saved_searches.push(SavedSearch {
                    id: SavedSearchId::new("s-1").expect("id"),
                    name: "Saved".to_owned(),
                    query: saved,
                });
            })
            .build();
        let view = View::SmartFolder(SavedSearchId::new("s-1").expect("id"));
        let filters = [FacetFilter::Folder(path("b")), FacetFilter::Tag(tag("y"))];
        let query = query_of(&view, &filters, &moment(), &catalog).expect("query");
        assert_eq!(query.folder, Some(path("b")));
        assert_eq!(query.tags.all.len(), 2);
        assert_eq!(paths(&documents(&catalog, &view, &filters, "", &moment(), &Matches::default())), ["b/two.pdf"]);
    }

    #[test]
    fn the_type_filter_and_years_narrow_the_documents() {
        let catalog = TestCatalog::new()
            .document("a.pdf", |document| {
                document.dated(date(2016, 5, 1));
            })
            .document("b.png", |document| {
                document.dated(date(2016, 5, 1));
            })
            .document("c.pdf", |document| {
                document.dated(date(2018, 1, 1));
            })
            .build();
        let filters = [
            FacetFilter::Type(dms_domain::document::DocumentKind::Pdf),
            FacetFilter::Year(YearFilter { year: 2017, and_earlier: true }),
        ];
        assert_eq!(paths(&documents(&catalog, &all(), &filters, "", &moment(), &Matches::default())), ["a.pdf"]);
    }

    #[test]
    fn the_tax_facet_narrows_to_the_year_or_one_of_its_areas() {
        use crate::features::library::model::TaxFacet;
        use dms_domain::tax::TaxArea;
        let catalog = TestCatalog::new()
            .document("a.pdf", |document| {
                document.tax(2025, TaxArea::Donations, None);
            })
            .document("b.pdf", |document| {
                document.tax(2025, TaxArea::Income, None);
            })
            .document("c.pdf", |document| {
                document.tags(&["Tax"]).dated(date(2025, 1, 1));
            })
            .build();
        let shown = |area| {
            let filters = [FacetFilter::Tax(TaxFacet { year: 2025, area })];
            paths(&documents(&catalog, &all(), &filters, "", &moment(), &Matches::default()))
        };

        assert_eq!(shown(None), ["a.pdf", "b.pdf"]);
        assert_eq!(shown(Some(TaxArea::Income)), ["b.pdf"]);
    }

    #[test]
    fn titles_name_the_view() {
        let catalog = TestCatalog::new().build();
        assert_eq!(title_of(&all(), &catalog), "All documents");
        assert_eq!(title_of(&View::Recent, &catalog), "Added this month");
        assert_eq!(title_of(&View::SmartFolder(SavedSearchId::new("gone").expect("id")), &catalog), "Saved view");
    }
}
