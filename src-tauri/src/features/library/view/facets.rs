//! The values each facet chip offers, with how many documents of the view have them (Kotlin `FacetOptions`).

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

use dms_domain::bill::BillStatus;
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentKind};
use dms_domain::tax::TaxArea;
use dms_domain::taxonomy::CategoryId;
use dms_domain::text;

use super::filter::year_of;
use super::tokens::{kind_label, payment_label};
use super::trees;
use crate::features::library::model::{Facet, FacetFilter, Moment, TaxFacet, YearFilter};

/// One value a facet chip offers; `depth` indents subcategories and subfolders, `missing` marks "Without a category".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacetOption {
    pub filter: FacetFilter,
    pub label: String,
    pub count: usize,
    pub depth: usize,
    pub missing: bool,
}

impl FacetOption {
    fn new(filter: FacetFilter, label: impl Into<String>, count: usize) -> Self {
        Self { filter, label: label.into(), count, depth: 0, missing: false }
    }
}

pub fn of(documents: &[&Document], catalog: &Catalog, moment: &Moment) -> Vec<(Facet, Vec<FacetOption>)> {
    Facet::ALL
        .into_iter()
        .map(|facet| {
            let options = match facet {
                Facet::Category => categories(documents, catalog),
                Facet::Tag => tags(documents, catalog),
                Facet::Year => years(documents, moment),
                Facet::Correspondent => correspondents(documents),
                Facet::Type => kinds(documents),
                Facet::Payment => payments(documents, catalog, moment),
                Facet::Folder => folders(documents),
                Facet::Tax => tax_years(documents),
            };
            (facet, options)
        })
        .collect()
}

/// The category tree, each count with its subcategories, then "Without a category".
fn categories(documents: &[&Document], catalog: &Catalog) -> Vec<FacetOption> {
    let mut counts: HashMap<&CategoryId, usize> = HashMap::new();
    for document in documents {
        for id in self_and_parents(document.category_id.as_ref(), catalog) {
            *counts.entry(id).or_default() += 1;
        }
    }
    let mut options: Vec<FacetOption> = trees::categories(catalog.categories().values(), documents.iter().copied())
        .into_iter()
        .map(|entry| FacetOption {
            depth: entry.depth,
            ..FacetOption::new(
                FacetFilter::Category(Some(entry.category.id.clone())),
                entry.category.name.clone(),
                counts.get(&entry.category.id).copied().unwrap_or(0),
            )
        })
        .collect();
    let without = documents.iter().filter(|document| document.category_id.is_none()).count();
    options.push(FacetOption {
        missing: true,
        ..FacetOption::new(FacetFilter::Category(None), "Without a category", without)
    });
    options
}

/// Every tag of the catalog, also unused ones, by how often it is used.
fn tags(documents: &[&Document], catalog: &Catalog) -> Vec<FacetOption> {
    let mut counts = HashMap::new();
    for tag in documents.iter().flat_map(|document| &document.tags) {
        *counts.entry(tag).or_insert(0) += 1;
    }
    let options = catalog
        .tags()
        .iter()
        .map(|tag| FacetOption::new(FacetFilter::Tag(tag.clone()), tag.value(), counts.get(tag).copied().unwrap_or(0)));
    by_count(options.collect())
}

fn years(documents: &[&Document], moment: &Moment) -> Vec<FacetOption> {
    let mut counts: HashMap<i32, usize> = HashMap::new();
    for document in documents {
        *counts.entry(year_of(document, moment)).or_default() += 1;
    }
    let mut years: Vec<i32> = counts.keys().copied().collect();
    years.sort_unstable_by(|first, second| second.cmp(first));
    years
        .into_iter()
        .map(|year| {
            FacetOption::new(
                FacetFilter::Year(YearFilter { year, and_earlier: false }),
                year.to_string(),
                counts[&year],
            )
        })
        .collect()
}

/// The correspondents grouped ignoring case, the first spelling shown, by how often they occur.
fn correspondents(documents: &[&Document]) -> Vec<FacetOption> {
    let mut groups: Vec<(String, String, usize)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for name in documents.iter().filter_map(|document| document.metadata.correspondent.as_deref().map(text::trim)) {
        let key = text::lowercase(name);
        if let Some(&at) = index.get(&key) {
            groups[at].2 += 1;
        } else {
            index.insert(key.clone(), groups.len());
            groups.push((key, name.to_owned(), 1));
        }
    }
    by_count(
        groups
            .into_iter()
            .map(|(_, name, count)| FacetOption::new(FacetFilter::Correspondent(name.clone()), name, count))
            .collect(),
    )
}

fn kinds(documents: &[&Document]) -> Vec<FacetOption> {
    [DocumentKind::Pdf, DocumentKind::Image, DocumentKind::Office, DocumentKind::Other]
        .into_iter()
        .filter_map(|kind| {
            let count = documents.iter().filter(|document| document.kind() == kind).count();
            (count > 0).then(|| FacetOption::new(FacetFilter::Type(kind), kind_label(kind), count))
        })
        .collect()
}

/// Open, overdue and paid bills; documents that are no bills have no payment state.
fn payments(documents: &[&Document], catalog: &Catalog, moment: &Moment) -> Vec<FacetOption> {
    [BillStatus::Open, BillStatus::Overdue, BillStatus::Paid]
        .into_iter()
        .filter_map(|status| {
            let today = moment.today();
            let count = documents
                .iter()
                .filter(|document| BillStatus::of(document, catalog.settings(), today) == Some(status))
                .count();
            (count > 0).then(|| FacetOption::new(FacetFilter::Payment(status), payment_label(status), count))
        })
        .collect()
}

fn folders(documents: &[&Document]) -> Vec<FacetOption> {
    trees::folders(documents.iter().copied(), &[], None)
        .into_iter()
        .filter(|entry| !entry.path.is_root())
        .map(|entry| FacetOption {
            depth: entry.depth - 1,
            ..FacetOption::new(FacetFilter::Folder(entry.path), entry.name, entry.document_count)
        })
        .collect()
}

/// Every tax year with assigned documents, newest first, each followed by its areas.
fn tax_years(documents: &[&Document]) -> Vec<FacetOption> {
    let mut counts: BTreeMap<(Reverse<i32>, Option<usize>), usize> = BTreeMap::new();
    for tax in documents.iter().filter_map(|document| document.tax.as_ref()) {
        let area = TaxArea::ALL.iter().position(|area| *area == tax.area);
        *counts.entry((Reverse(tax.year), None)).or_default() += 1;
        *counts.entry((Reverse(tax.year), area)).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|((Reverse(year), area), count)| {
            let facet = TaxFacet { year, area: area.map(|index| TaxArea::ALL[index]) };
            let label = facet.area.map_or_else(|| year.to_string(), |area| area.label().to_owned());
            FacetOption {
                depth: usize::from(facet.area.is_some()),
                ..FacetOption::new(FacetFilter::Tax(facet), label, count)
            }
        })
        .collect()
}

/// By count, highest first, then by label ignoring case; equal ones keep their order.
fn by_count(mut options: Vec<FacetOption>) -> Vec<FacetOption> {
    options.sort_by(|first, second| {
        second
            .count
            .cmp(&first.count)
            .then_with(|| text::compare(&text::lowercase(&first.label), &text::lowercase(&second.label)))
    });
    options
}

fn self_and_parents<'a>(id: Option<&'a CategoryId>, catalog: &'a Catalog) -> HashSet<&'a CategoryId> {
    let mut chain = HashSet::new();
    let mut current = id;
    while let Some(category) = current {
        if !chain.insert(category) {
            break;
        }
        current = catalog.categories().get(category).and_then(|it| it.parent_id.as_ref());
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, date, moment};

    fn labels(options: &[FacetOption]) -> Vec<(String, usize)> {
        options.iter().map(|option| (option.label.clone(), option.count)).collect()
    }

    fn facet(catalog: &Catalog, wanted: Facet) -> Vec<FacetOption> {
        let documents: Vec<&Document> = catalog.documents().values().collect();
        of(&documents, catalog, &moment()).into_iter().find(|(facet, _)| *facet == wanted).expect("facet").1
    }

    fn sample() -> Catalog {
        TestCatalog::new()
            .category("c-home", "Home", None)
            .category("c-energy", "Energy", Some("c-home"))
            .document("Home/Power/a.pdf", |document| {
                document.category("c-energy").tags(&["b", "a"]).correspondent("City Power").dated(date(2024, 3, 1));
            })
            .document("Home/b.png", |document| {
                document.category("c-home").tags(&["a"]).correspondent(" city power ").dated(date(2025, 1, 1));
            })
            .document("c.pdf", |document| {
                document.correspondent("Bank").dated(date(2025, 2, 1));
            })
            .build()
    }

    #[test]
    fn category_counts_include_subcategories_and_end_with_without_a_category() {
        let options = facet(&sample(), Facet::Category);
        assert_eq!(
            labels(&options),
            [("Home".to_owned(), 2), ("Energy".to_owned(), 1), ("Without a category".to_owned(), 1)]
        );
        assert_eq!(options[1].depth, 1);
        assert!(options[2].missing);
    }

    #[test]
    fn tags_and_correspondents_are_sorted_by_count_then_name() {
        assert_eq!(labels(&facet(&sample(), Facet::Tag)), [("a".to_owned(), 2), ("b".to_owned(), 1)]);
        assert_eq!(
            labels(&facet(&sample(), Facet::Correspondent)),
            [("City Power".to_owned(), 2), ("Bank".to_owned(), 1)]
        );
    }

    #[test]
    fn years_are_newest_first_and_kinds_only_present_ones() {
        assert_eq!(labels(&facet(&sample(), Facet::Year)), [("2025".to_owned(), 2), ("2024".to_owned(), 1)]);
        assert_eq!(labels(&facet(&sample(), Facet::Type)), [("PDF".to_owned(), 2), ("Image".to_owned(), 1)]);
    }

    #[test]
    fn tax_years_are_newest_first_each_followed_by_its_areas() {
        let catalog = TestCatalog::new()
            .document("a.pdf", |document| {
                document.tax(2024, TaxArea::Donations, None);
            })
            .document("b.pdf", |document| {
                document.tax(2025, TaxArea::HealthCosts, None);
            })
            .document("c.pdf", |document| {
                document.tax(2025, TaxArea::Income, None);
            })
            .document("d.pdf", |_| {})
            .build();

        let options = facet(&catalog, Facet::Tax);

        assert_eq!(
            labels(&options),
            [
                ("2025".to_owned(), 2),
                ("Income".to_owned(), 1),
                ("Health costs".to_owned(), 1),
                ("2024".to_owned(), 1),
                ("Donations".to_owned(), 1),
            ]
        );
        assert_eq!(options[1].filter, FacetFilter::Tax(TaxFacet { year: 2025, area: Some(TaxArea::Income) }));
        assert_eq!((options[0].depth, options[1].depth), (0, 1));
    }

    #[test]
    fn folders_are_a_tree_without_the_root() {
        let options = facet(&sample(), Facet::Folder);
        assert_eq!(labels(&options), [("Home".to_owned(), 2), ("Power".to_owned(), 1)]);
        assert_eq!(options[1].depth, 1);
    }

    #[test]
    fn the_payment_facet_counts_open_overdue_and_paid_bills_and_the_filter_shows_them() {
        use dms_domain::field::FieldPresets;

        use crate::features::library::model::View;
        use crate::features::library::test_catalog::path;
        use crate::features::library::view::filter::{Matches, documents};

        let bill = |due: chrono::NaiveDate, paid: bool| {
            move |document: &mut crate::features::library::test_catalog::DocumentBuilder| {
                document.money("amount", "12.30").field_date("due", due);
                if paid {
                    document.field_date("paid-on", due);
                }
            }
        };
        let today = moment().today();
        let catalog = TestCatalog::new()
            .settings(|settings| {
                settings.field_definitions =
                    vec![FieldPresets::amount(), FieldPresets::due_date(), FieldPresets::paid_on()];
            })
            .document("open.pdf", bill(today + chrono::Days::new(5), false))
            .document("late.pdf", bill(today - chrono::Days::new(5), false))
            .document("paid.pdf", bill(today - chrono::Days::new(9), true))
            .document("letter.pdf", |_| {})
            .build();

        let late = documents(
            &catalog,
            &View::Folder(path("")),
            &[FacetFilter::Payment(BillStatus::Overdue)],
            "",
            &moment(),
            &Matches::default(),
        );

        assert_eq!(
            labels(&facet(&catalog, Facet::Payment)),
            [("Open".to_owned(), 1), ("Overdue".to_owned(), 1), ("Paid".to_owned(), 1)]
        );
        assert_eq!(late.iter().map(|document| document.path.value()).collect::<Vec<_>>(), ["late.pdf"]);
    }
}
