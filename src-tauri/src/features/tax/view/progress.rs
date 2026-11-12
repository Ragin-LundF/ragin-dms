//! The dashboard card "Tax year progress".

use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::tax::TaxItemState;

use super::amounts::default_year;
use super::checklist::Checklist;
use crate::features::tax::dto::TaxProgressDto;

/// The progress of the default year; `None` for a library that uses no tax assignment and no checklist yet, so the
/// card stays hidden there.
pub fn present_progress(catalog: &Catalog, today: NaiveDate) -> Option<TaxProgressDto> {
    let uses_tax =
        !catalog.tax_years().is_empty() || catalog.documents().values().any(|document| document.tax.is_some());
    if !uses_tax {
        return None;
    }
    let year = default_year(today);
    let checklist = Checklist::of(catalog, year, today);
    let (found, total) = checklist.progress();
    let missing = checklist.year.items.iter().filter(|item| item.state == TaxItemState::Missing).count();
    Some(TaxProgressDto { year, found, total, missing: u32::try_from(missing).unwrap_or(u32::MAX) })
}

#[cfg(test)]
mod tests {
    use dms_domain::tax::TaxArea;

    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, date};

    #[test]
    fn the_card_counts_the_default_year_and_hides_without_tax_use() {
        let plain = TestCatalog::new().document("a.pdf", |_| {}).build();
        let taxed = TestCatalog::new()
            .document("a.pdf", |document| {
                document.title("Lohnsteuerbescheinigung 2025").tax(2025, TaxArea::Income, None);
            })
            .build();

        assert_eq!(present_progress(&plain, date(2026, 3, 1)), None);
        assert_eq!(
            present_progress(&taxed, date(2026, 3, 1)),
            Some(TaxProgressDto { year: 2025, found: 1, total: 2, missing: 1 })
        );
    }
}
