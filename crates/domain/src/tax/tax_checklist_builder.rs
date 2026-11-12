use chrono::{Months, NaiveDate};

use crate::catalog::Catalog;
use crate::document::{Document, DocumentId};
use crate::expectation::RhythmDetector;
use crate::tax::{TaxArea, TaxChecklistItem, TaxItemId, TaxItemState, TaxYear};
use crate::text;

/// Builds the checklist of a tax year from what the library knows. Pure.
///
/// The list holds (a) a standard item per area of [`Self::STANDARD`], (b) every sender and area with a document in
/// the year before, (c) every sender with a tax history whose next document is expected between the start of the
/// year and the end of July after it, and (d) every sender and area with a document of the year itself. An item is
/// [`TaxItemState::Found`] when a document of the year in its area is linked or matches (same sender, or for an item
/// without a sender: a title containing its label), [`TaxItemState::Expected`] while its usual date is still ahead,
/// and [`TaxItemState::Missing`] otherwise. Existing items keep their id, label and note; items marked
/// [`TaxItemState::NotNeeded`] stay so, and no item is ever removed.
pub struct TaxChecklistBuilder;

/// One item the library suggests; `usual` is when its document usually arrives.
struct Candidate {
    area: TaxArea,
    label: String,
    correspondent: Option<String>,
    usual: Option<NaiveDate>,
}

impl Candidate {
    fn is_for(&self, item: &TaxChecklistItem) -> bool {
        item.area == self.area
            && match (&self.correspondent, &item.correspondent) {
                (Some(wanted), Some(known)) => key(wanted) == key(known),
                (None, None) => !item.added_by_user,
                _ => false,
            }
    }
}

impl TaxChecklistBuilder {
    /// The items every tax return usually needs; at most one per area, so a renamed one is still recognized.
    pub const STANDARD: [(TaxArea, &'static str); 2] =
        [(TaxArea::Income, "Lohnsteuerbescheinigung"), (TaxArea::PensionInsurance, "Health insurance certificate")];

    /// The checklist of `year` on `today`, built on the stored one; `new_id` names the items it adds.
    #[must_use]
    pub fn build(catalog: &Catalog, year: i32, today: NaiveDate, new_id: &mut dyn FnMut() -> TaxItemId) -> TaxYear {
        let mut tax_year = catalog.tax_years().get(&year).cloned().unwrap_or_else(|| TaxYear::new(year));
        let candidates = candidates(catalog, year, today);
        for candidate in &candidates {
            if !tax_year.items.iter().any(|item| candidate.is_for(item)) {
                tax_year.items.push(TaxChecklistItem {
                    id: new_id(),
                    area: candidate.area,
                    label: candidate.label.clone(),
                    correspondent: candidate.correspondent.clone(),
                    document_id: None,
                    state: TaxItemState::Missing,
                    note: None,
                    added_by_user: false,
                });
            }
        }
        let documents: Vec<&Document> = documents_of(catalog, year).collect();
        for item in &mut tax_year.items {
            let usual =
                candidates.iter().find(|candidate| candidate.is_for(item)).and_then(|candidate| candidate.usual);
            refresh(item, &documents, usual, today);
        }
        tax_year
    }
}

fn candidates(catalog: &Catalog, year: i32, today: NaiveDate) -> Vec<Candidate> {
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut add = |candidate: Candidate| {
        let known = candidates.iter_mut().find(|known| {
            known.area == candidate.area
                && known.correspondent.as_deref().map(key) == candidate.correspondent.as_deref().map(key)
        });
        match known {
            Some(known) => known.usual = known.usual.or(candidate.usual),
            None => candidates.push(candidate),
        }
    };
    let end_of_february = NaiveDate::from_ymd_opt(year + 1, 3, 1).map(|march| march.pred_opt().unwrap_or(march));
    for (area, label) in TaxChecklistBuilder::STANDARD {
        add(Candidate { area, label: label.to_owned(), correspondent: None, usual: end_of_february });
    }
    for (area, correspondent, last) in senders_of(catalog, year - 1) {
        add(Candidate {
            area,
            label: correspondent.clone(),
            correspondent: Some(correspondent),
            usual: next_year(last),
        });
    }
    for (area, correspondent, expected_on) in expected_senders(catalog, year, today) {
        add(Candidate {
            area,
            label: correspondent.clone(),
            correspondent: Some(correspondent),
            usual: Some(expected_on),
        });
    }
    for (area, correspondent, _) in senders_of(catalog, year) {
        add(Candidate { area, label: correspondent.clone(), correspondent: Some(correspondent), usual: None });
    }
    candidates
}

/// Every sender and area with a document of `year`, the name as the latest document spells it, and the latest date.
fn senders_of(catalog: &Catalog, year: i32) -> Vec<(TaxArea, String, Option<NaiveDate>)> {
    let mut senders: Vec<(TaxArea, String, Option<NaiveDate>)> = Vec::new();
    for document in documents_of(catalog, year) {
        let (Some(tax), Some(name)) = (&document.tax, correspondent_of(document)) else {
            continue;
        };
        let date = document.metadata.document_date;
        match senders.iter_mut().find(|(area, known, _)| *area == tax.area && key(known) == key(name)) {
            Some(sender) if date > sender.2 => *sender = (tax.area, name.to_owned(), date),
            Some(_) => {}
            None => senders.push((tax.area, name.to_owned(), date)),
        }
    }
    senders
}

/// The senders with a rhythm whose next document is due from 1 January of `year` to 31 July after it, in the area
/// of their latest document with a tax assignment; senders without one are not tax relevant.
fn expected_senders(catalog: &Catalog, year: i32, today: NaiveDate) -> Vec<(TaxArea, String, NaiveDate)> {
    let (Some(start), Some(end)) = (NaiveDate::from_ymd_opt(year, 1, 1), NaiveDate::from_ymd_opt(year + 1, 7, 31))
    else {
        return Vec::new();
    };
    RhythmDetector::expected(catalog, today)
        .into_iter()
        .filter(|expected| (start..=end).contains(&expected.expected_on))
        .filter_map(|expected| {
            let area = latest_area_of(catalog, &expected.correspondent)?;
            Some((area, expected.correspondent, expected.expected_on))
        })
        .collect()
}

fn latest_area_of(catalog: &Catalog, correspondent: &str) -> Option<TaxArea> {
    catalog
        .documents()
        .values()
        .filter(|document| correspondent_of(document).is_some_and(|name| key(name) == key(correspondent)))
        .filter_map(|document| document.tax.as_ref().map(|tax| (tax.year, document.metadata.document_date, tax.area)))
        .max_by_key(|(year, date, _)| (*year, *date))
        .map(|(_, _, area)| area)
}

fn refresh(item: &mut TaxChecklistItem, documents: &[&Document], usual: Option<NaiveDate>, today: NaiveDate) {
    if item.state == TaxItemState::NotNeeded {
        return;
    }
    let linked = item.document_id.as_ref().filter(|id| documents.iter().any(|document| belongs(document, item, id)));
    item.document_id = linked.cloned().or_else(|| first_match(item, documents));
    item.state = match (&item.document_id, usual) {
        (Some(_), _) => TaxItemState::Found,
        (None, Some(usual)) if usual >= today => TaxItemState::Expected,
        (None, _) => TaxItemState::Missing,
    };
}

/// `true` if the document `id` is among the year's documents and in the item's area.
fn belongs(document: &Document, item: &TaxChecklistItem, id: &DocumentId) -> bool {
    &document.id == id && document.tax.as_ref().is_some_and(|tax| tax.area == item.area)
}

/// The earliest document of the item's area from its sender, or for an item without one, with its label in the title.
fn first_match(item: &TaxChecklistItem, documents: &[&Document]) -> Option<DocumentId> {
    let label = key(&item.label);
    documents
        .iter()
        .filter(|document| document.tax.as_ref().is_some_and(|tax| tax.area == item.area))
        .filter(|document| match &item.correspondent {
            Some(wanted) => correspondent_of(document).is_some_and(|name| key(name) == key(wanted)),
            None => document.metadata.title.as_deref().is_some_and(|title| key(title).contains(&label)),
        })
        .min_by_key(|document| (document.metadata.document_date, document.path.value().to_owned()))
        .map(|document| document.id.clone())
}

fn documents_of(catalog: &Catalog, year: i32) -> impl Iterator<Item = &Document> {
    catalog.documents().values().filter(move |document| document.tax.as_ref().is_some_and(|tax| tax.year == year))
}

fn correspondent_of(document: &Document) -> Option<&str> {
    document.metadata.correspondent.as_deref().map(text::trim).filter(|name| !name.is_empty())
}

fn next_year(date: Option<NaiveDate>) -> Option<NaiveDate> {
    date.and_then(|date| date.checked_add_months(Months::new(12)))
}

fn key(name: &str) -> String {
    text::lowercase(text::trim(name))
}
