//! A sentence's translation as the library's filters.

use dms_application::search::QueryTranslation;

use super::dto::PlainSearchResultDto;
use crate::features::library::dto::params::LibraryFilterDto;
use crate::features::library::model::{FacetFilter, TaxFacet};
use crate::features::library::view::tokens::year_of;

/// The filters of `translation`, one per condition: a date range that covers one year, or ends with one, becomes the
/// year filter; any other range stays a date filter. A payment state becomes the payment filter, a tax year (with
/// its area) the tax filter; a tax area without a year has no filter and counts as not understood.
pub fn result_dto(translation: &QueryTranslation) -> PlainSearchResultDto {
    let query = &translation.query;
    let category = query.category.clone().map(|id| FacetFilter::Category(Some(id)));
    let tags = query.tags.all.iter().cloned().map(FacetFilter::Tag);
    let correspondent = query.correspondent.clone().map(FacetFilter::Correspondent);
    let dates = query.dates.map(|range| year_of(range).map_or(FacetFilter::Dates(range), FacetFilter::Year));
    let fields = query.fields.iter().cloned().map(FacetFilter::Field);
    let payment = translation.payment.map(FacetFilter::Payment);
    let tax = translation.tax_year.map(|year| FacetFilter::Tax(TaxFacet { year, area: translation.tax_area }));
    let filters =
        category.into_iter().chain(tags).chain(correspondent).chain(dates).chain(fields).chain(payment).chain(tax);
    let mut not_understood = translation.not_understood.clone();
    if let (None, Some(area)) = (translation.tax_year, translation.tax_area) {
        not_understood.push(area.label().to_owned());
    }
    PlainSearchResultDto {
        filters: filters.map(|filter| LibraryFilterDto::of(&filter)).collect(),
        text: query.text.clone().unwrap_or_default(),
        not_understood,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::dto::params::{LibraryComparisonDto, LibraryPaymentDto};
    use crate::features::settings::dto::FieldTypeDto;
    use crate::features::tax::dto::TaxAreaDto;
    use chrono::NaiveDate;
    use dms_domain::bill::BillStatus;
    use dms_domain::field::{FieldKey, FieldValues};
    use dms_domain::query::{DateRange, DocumentQuery, FieldComparison, FieldCondition};
    use dms_domain::tax::TaxArea;
    use dms_domain::taxonomy::{CategoryId, TagName};

    fn date(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(year, month, day)
    }

    fn translation(query: DocumentQuery) -> QueryTranslation {
        QueryTranslation {
            query,
            payment: Some(BillStatus::Open),
            tax_year: Some(2025),
            tax_area: Some(TaxArea::HouseholdServices),
            not_understood: vec!["Nordlicht".to_owned()],
            model: "m".to_owned(),
        }
    }

    #[test]
    fn every_condition_becomes_one_filter_and_a_whole_year_becomes_the_year_filter() {
        let amount = FieldValues::parse(dms_domain::field::FieldType::Money, "50 EUR", None);
        let mut query = DocumentQuery {
            text: Some("heating".to_owned()),
            category: Some(CategoryId::new("c-bills").expect("id")),
            correspondent: Some("Stadtwerke Beispielstadt".to_owned()),
            dates: Some(DateRange::year(2025).expect("year")),
            fields: vec![
                FieldCondition::new(FieldKey::new("amount").expect("key"), FieldComparison::GreaterThan, amount)
                    .expect("condition"),
            ],
            ..DocumentQuery::default()
        };
        query.tags.all.insert(TagName::of("Tax").expect("tag"));

        let result = result_dto(&translation(query));

        assert_eq!(
            result.filters,
            vec![
                LibraryFilterDto::Category { category_id: Some("c-bills".to_owned()) },
                LibraryFilterDto::Tag { tag: "Tax".to_owned() },
                LibraryFilterDto::Correspondent { name: "Stadtwerke Beispielstadt".to_owned() },
                LibraryFilterDto::Year { year: 2025, and_earlier: false },
                LibraryFilterDto::Field {
                    key: "amount".to_owned(),
                    comparison: LibraryComparisonDto::GreaterThan,
                    value: Some("50 EUR".to_owned()),
                    value_type: Some(FieldTypeDto::Money),
                },
                LibraryFilterDto::Payment { status: LibraryPaymentDto::Open },
                LibraryFilterDto::Tax { year: 2025, area: Some(TaxAreaDto::HouseholdServices) },
            ]
        );
        assert_eq!(result.text, "heating");
        assert_eq!(result.not_understood, ["Nordlicht"]);
        let parsed: Vec<_> = result.filters.iter().map(LibraryFilterDto::parse).collect();
        assert!(parsed.iter().all(Result::is_ok), "the window can send every filter back: {parsed:?}");
    }

    #[test]
    fn a_range_that_is_no_whole_year_stays_a_date_filter() {
        let since =
            DocumentQuery { dates: Some(DateRange { from: date(2024, 1, 1), to: None }), ..DocumentQuery::default() };

        let result = result_dto(&translation(since));

        assert_eq!(
            result.filters,
            vec![
                LibraryFilterDto::Dates { from: Some("2024-01-01".to_owned()), to: None },
                LibraryFilterDto::Payment { status: LibraryPaymentDto::Open },
                LibraryFilterDto::Tax { year: 2025, area: Some(TaxAreaDto::HouseholdServices) },
            ]
        );
    }

    #[test]
    fn a_tax_area_without_a_year_is_no_filter_but_not_understood() {
        let area_only = QueryTranslation { payment: None, tax_year: None, ..translation(DocumentQuery::default()) };

        let result = result_dto(&area_only);

        assert_eq!(result.filters, Vec::new());
        assert_eq!(result.not_understood, ["Nordlicht".to_owned(), TaxArea::HouseholdServices.label().to_owned()]);
    }
}
