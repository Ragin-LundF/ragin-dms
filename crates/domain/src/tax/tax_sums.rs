use indexmap::IndexMap;

use crate::catalog::Catalog;
use crate::document::Document;
use crate::field::FieldPresets;
use crate::money::{Money, MoneyTotals};
use crate::tax::TaxArea;

/// What the documents of a tax year add up to per area.
pub struct TaxSums;

impl TaxSums {
    /// The deductible part of a document with a tax assignment: the one set on the assignment, or else the
    /// document's amount field.
    #[must_use]
    pub fn deductible(document: &Document) -> Option<Money> {
        let tax = document.tax.as_ref()?;
        tax.deductible.clone().or_else(|| document.fields.get(&FieldPresets::amount().key).and_then(Money::of_field))
    }

    /// The sums of the deductible parts of `year`'s documents, per area in the order of [`TaxArea::ALL`]; areas
    /// without an amount are left out.
    #[must_use]
    pub fn of(catalog: &Catalog, year: i32) -> IndexMap<TaxArea, MoneyTotals> {
        let mut sums: IndexMap<TaxArea, MoneyTotals> = IndexMap::new();
        for area in TaxArea::ALL {
            let totals = catalog
                .documents()
                .values()
                .filter(|document| document.tax.as_ref().is_some_and(|tax| tax.year == year && tax.area == area))
                .filter_map(Self::deductible)
                .fold(MoneyTotals::default(), |totals, money| totals.plus(&money));
            if !totals.is_empty() {
                sums.insert(area, totals);
            }
        }
        sums
    }
}
