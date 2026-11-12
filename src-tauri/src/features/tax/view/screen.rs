//! The "Tax years" screen: the checklist of a year by area, progress and the amounts per area.

use std::collections::BTreeSet;

use chrono::{Datelike, NaiveDate};
use dms_domain::catalog::Catalog;
use dms_domain::tax::{TaxArea, TaxChecklistItem, TaxItemState, TaxSums};

use super::amounts::{area_option, area_options, default_year, money_display};
use super::checklist::Checklist;
use crate::features::tax::dto::{
    TaxAreaGroupDto, TaxAreaStatusDto, TaxItemDto, TaxItemStateDto, TaxSumDto, TaxYearViewDto,
};

pub fn present_tax_year(catalog: &Catalog, year: Option<i32>, today: NaiveDate) -> TaxYearViewDto {
    let year = year.unwrap_or_else(|| default_year(today));
    let checklist = Checklist::of(catalog, year, today);
    let (found, total) = checklist.progress();
    let areas = TaxArea::ALL
        .into_iter()
        .filter_map(|area| {
            let items: Vec<&TaxChecklistItem> = checklist.year.items.iter().filter(|item| item.area == area).collect();
            (!items.is_empty()).then(|| group_of(area, &items, catalog))
        })
        .collect();
    let sums = TaxSums::of(catalog, year)
        .into_iter()
        .map(|(area, totals)| TaxSumDto {
            area: area_option(area),
            amounts: totals.amounts().iter().map(money_display).collect(),
        })
        .collect();
    TaxYearViewDto {
        year,
        years: years_of(catalog, year, today),
        opened: checklist.stored,
        found,
        total,
        areas,
        sums,
        area_options: area_options(),
    }
}

/// The year shown, the default one, this year and last year, and every year with a checklist or an assigned
/// document, newest first.
fn years_of(catalog: &Catalog, shown: i32, today: NaiveDate) -> Vec<i32> {
    let assigned = catalog.documents().values().filter_map(|document| document.tax.as_ref().map(|tax| tax.year));
    let years: BTreeSet<i32> = catalog
        .tax_years()
        .keys()
        .copied()
        .chain(assigned)
        .chain([shown, default_year(today), today.year(), today.year() - 1])
        .collect();
    years.into_iter().rev().collect()
}

fn group_of(area: TaxArea, items: &[&TaxChecklistItem], catalog: &Catalog) -> TaxAreaGroupDto {
    let has = |state: TaxItemState| items.iter().any(|item| item.state == state);
    let status = if has(TaxItemState::Missing) {
        TaxAreaStatusDto::Missing
    } else if has(TaxItemState::Expected) {
        TaxAreaStatusDto::Expected
    } else {
        TaxAreaStatusDto::Complete
    };
    let count = |wanted: &dyn Fn(TaxItemState) -> bool| {
        u32::try_from(items.iter().filter(|item| wanted(item.state)).count()).unwrap_or(u32::MAX)
    };
    TaxAreaGroupDto {
        area: area_option(area),
        found: count(&|state| state == TaxItemState::Found),
        total: count(&|state| state != TaxItemState::NotNeeded),
        status,
        items: items.iter().map(|item| item_of(item, catalog)).collect(),
    }
}

fn item_of(item: &TaxChecklistItem, catalog: &Catalog) -> TaxItemDto {
    let document = item.document_id.as_ref().and_then(|id| catalog.documents().get(id));
    let document_name = document
        .map(|document| document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned()));
    TaxItemDto {
        id: item.id.value().to_owned(),
        label: item.label.clone(),
        correspondent: item.correspondent.clone(),
        state: state_dto(item.state),
        document_id: document.map(|document| document.id.value().to_owned()),
        detail: document_name.or_else(|| item.note.clone()),
        amount: document.and_then(TaxSums::deductible).as_ref().map(money_display),
        added_by_user: item.added_by_user,
    }
}

pub fn state_dto(state: TaxItemState) -> TaxItemStateDto {
    match state {
        TaxItemState::Found => TaxItemStateDto::Found,
        TaxItemState::Expected => TaxItemStateDto::Expected,
        TaxItemState::Missing => TaxItemStateDto::Missing,
        TaxItemState::NotNeeded => TaxItemStateDto::NotNeeded,
    }
}

#[cfg(test)]
mod tests {
    use dms_domain::tax::{TaxItemId, TaxYear};

    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, date};

    fn library() -> Catalog {
        TestCatalog::new()
            .document("2024/sweep.pdf", |document| {
                document.correspondent("Chimney Sweep").dated(date(2024, 11, 3)).tax(
                    2024,
                    TaxArea::HouseholdServices,
                    None,
                );
            })
            .document("2025/sweep.pdf", |document| {
                document
                    .title("Sweep 2025")
                    .correspondent("Chimney Sweep")
                    .dated(date(2025, 11, 2))
                    .money("amount", "96.00")
                    .tax(2025, TaxArea::HouseholdServices, None);
            })
            .document("2025/plumber.pdf", |document| {
                document.correspondent("Plumber").money("amount", "1200.00").tax(
                    2025,
                    TaxArea::HouseholdServices,
                    Some("684.50"),
                );
            })
            .build()
    }

    #[test]
    fn the_year_shows_its_areas_with_found_items_their_amounts_and_the_sums() {
        let view = present_tax_year(&library(), Some(2025), date(2026, 3, 10));

        assert!(!view.opened, "never stored: a preview");
        assert_eq!((view.found, view.total), (2, 4));
        let areas: Vec<(&str, TaxAreaStatusDto)> =
            view.areas.iter().map(|group| (group.area.label.as_str(), group.status)).collect();
        assert_eq!(
            areas,
            [
                ("Income", TaxAreaStatusDto::Missing),
                ("Insurance and pension", TaxAreaStatusDto::Missing),
                ("Household services and craftsmen", TaxAreaStatusDto::Complete),
            ]
        );
        let household = &view.areas[2];
        let sweep = household.items.iter().find(|item| item.label == "Chimney Sweep").expect("sweep");
        assert_eq!((sweep.detail.as_deref(), sweep.amount.as_deref()), (Some("Sweep 2025"), Some("€ 96.00")));
        let plumber = household.items.iter().find(|item| item.label == "Plumber").expect("plumber");
        assert_eq!(plumber.amount.as_deref(), Some("€ 684.50"), "the deductible part wins over the amount");
        assert_eq!(view.sums.len(), 1);
        assert_eq!(view.sums[0].amounts, ["€ 780.50"]);
        assert_eq!(view.area_options.len(), 9);
    }

    #[test]
    fn a_stored_year_is_shown_as_stored_and_not_needed_items_leave_the_total() {
        let mut year = TaxYear::new(2025);
        year.items.push(TaxChecklistItem {
            id: TaxItemId::new("t-1").expect("id"),
            area: TaxArea::Donations,
            label: "Receipt".to_owned(),
            correspondent: None,
            document_id: None,
            state: TaxItemState::NotNeeded,
            note: Some("No donations this year".to_owned()),
            added_by_user: true,
        });
        let catalog = match library().save_tax_year(year).expect("valid") {
            dms_domain::catalog::CatalogUpdate::Applied(catalog) => catalog,
            dms_domain::catalog::CatalogUpdate::Rejected(reason) => panic!("{reason:?}"),
        };

        let view = present_tax_year(&catalog, Some(2025), date(2026, 3, 10));

        assert!(view.opened);
        assert_eq!((view.found, view.total), (0, 0));
        let item = &view.areas[0].items[0];
        assert_eq!((item.state, item.detail.as_deref()), (TaxItemStateDto::NotNeeded, Some("No donations this year")));
        assert_eq!(view.areas[0].status, TaxAreaStatusDto::Complete);
    }

    #[test]
    fn the_picker_offers_the_default_and_every_used_year_newest_first() {
        let view = present_tax_year(&library(), None, date(2026, 3, 10));

        assert_eq!(view.year, 2025, "last year until the end of July");
        assert_eq!(view.years, [2026, 2025, 2024]);
    }
}
