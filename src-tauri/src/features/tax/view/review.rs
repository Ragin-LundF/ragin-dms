//! The tax rows of the review of an AI suggestion: year, area and the labour share, each checked on its own.

use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::field::{FieldType, FieldValues};
use dms_domain::money::Money;
use dms_domain::tax::{TaxArea, TaxAssignment};
use dms_domain::text;

use super::amounts::{area_labels, area_of_label, money_text};
use crate::features::document::dto::{ReviewRow, ReviewRowDraft};

const YEAR: &str = "taxYear";
const AREA: &str = "taxArea";
const DEDUCTIBLE: &str = "taxDeductible";

/// The rows for the suggested tax assignment, with the document's current one; none without a suggestion.
pub fn tax_rows(suggested: Option<&TaxAssignment>, document: &Document) -> Vec<ReviewRow> {
    let Some(tax) = suggested else {
        return Vec::new();
    };
    let current = document.tax.as_ref();
    let mut rows = vec![
        row(YEAR, "Tax year", tax.year.to_string(), current.map(|it| it.year.to_string())),
        ReviewRow {
            options: area_labels(),
            ..row(AREA, "Tax area", tax.area.label().to_owned(), current.map(|it| it.area.label().to_owned()))
        },
    ];
    if let Some(deductible) = &tax.deductible {
        let known = current.and_then(|it| it.deductible.as_ref()).map(money_text);
        rows.push(row(DEDUCTIBLE, "Deductible part (labour share)", money_text(deductible), known));
    }
    rows
}

/// The approved tax assignment of the checked rows: `None` while year and area are both unchecked, an error when
/// only one of them is checked or a value does not fit.
pub fn approved_tax(catalog: &Catalog, rows: &[ReviewRowDraft]) -> Result<Option<TaxAssignment>, String> {
    let taken = |key: &str| {
        rows.iter()
            .find(|row| row.key == key && row.accepted)
            .map(|row| text::trim(&row.value).to_owned())
            .filter(|value| !value.is_empty())
    };
    let (year, area) = match (taken(YEAR), taken(AREA)) {
        (None, None) => return Ok(None),
        (Some(year), Some(area)) => (year, area),
        _ => return Err("A tax assignment needs both a year and an area".to_owned()),
    };
    let year = year.parse::<i32>().map_err(|_| format!("\"{year}\" is not a tax year"))?;
    let area = area_of_label(&area).ok_or_else(|| format!("\"{area}\" is not a tax area"))?;
    let currency = catalog.settings().layout.default_currency.as_ref();
    let deductible = taken(DEDUCTIBLE)
        .map(|raw| {
            FieldValues::parse(FieldType::Money, &raw, currency)
                .as_ref()
                .and_then(Money::of_field)
                .ok_or_else(|| format!("\"{raw}\" is not an amount"))
        })
        .transpose()?;
    let tax = TaxAssignment { year, area, deductible: deductible.filter(|_| area == TaxArea::HouseholdServices) };
    tax.validate().map_err(|error| error.to_string())?;
    Ok(Some(tax))
}

fn row(key: &str, label: &str, value: String, current: Option<String>) -> ReviewRow {
    ReviewRow {
        key: key.to_owned(),
        label: label.to_owned(),
        accepted: true,
        value,
        current: current.unwrap_or_default(),
        options: Vec::new(),
        hint: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::test_catalog::TestCatalog;

    fn draft(key: &str, value: &str) -> ReviewRowDraft {
        ReviewRowDraft { key: key.to_owned(), accepted: true, value: value.to_owned() }
    }

    fn library() -> (Catalog, Document) {
        let catalog = TestCatalog::new()
            .settings(|settings| {
                settings.layout.default_currency = Some(dms_domain::field::CurrencyCode::new("EUR").expect("currency"));
            })
            .document("Inbox/plumber.pdf", |_| {})
            .build();
        let letter = catalog.documents().values().next().cloned().expect("document");
        (catalog, letter)
    }

    #[test]
    fn rows_show_year_area_and_labour_share_with_the_current_values() {
        let (catalog, mut letter) = library();
        letter.tax = Some(TaxAssignment { year: 2024, area: TaxArea::Other, deductible: None });
        let currency = catalog.settings().layout.default_currency.as_ref();
        let labour = FieldValues::parse(FieldType::Money, "684.50", currency).as_ref().and_then(Money::of_field);
        let suggested = TaxAssignment { year: 2025, area: TaxArea::HouseholdServices, deductible: labour };

        let rows = tax_rows(Some(&suggested), &letter);

        let shown: Vec<(&str, &str, &str)> =
            rows.iter().map(|row| (row.key.as_str(), row.value.as_str(), row.current.as_str())).collect();
        assert_eq!(
            shown,
            [
                ("taxYear", "2025", "2024"),
                ("taxArea", "Household services and craftsmen", "Other"),
                ("taxDeductible", "684.50 EUR", ""),
            ]
        );
        assert_eq!(rows[1].options.len(), TaxArea::ALL.len());
        assert_eq!(tax_rows(None, &letter), Vec::new());
    }

    #[test]
    fn approval_needs_year_and_area_and_checks_the_values() {
        let (catalog, _) = library();
        let approved = approved_tax(
            &catalog,
            &[
                draft("taxYear", " 2025 "),
                draft("taxArea", "household SERVICES and craftsmen"),
                draft("taxDeductible", "684,50"),
            ],
        )
        .expect("valid")
        .expect("an assignment");
        assert_eq!((approved.year, approved.area), (2025, TaxArea::HouseholdServices));
        assert_eq!(approved.deductible.map(|money| money_text(&money)).as_deref(), Some("684.50 EUR"));

        assert_eq!(approved_tax(&catalog, &[]), Ok(None));
        assert_eq!(
            approved_tax(&catalog, &[draft("taxYear", "2025")]),
            Err("A tax assignment needs both a year and an area".to_owned())
        );
        assert_eq!(
            approved_tax(&catalog, &[draft("taxYear", "next"), draft("taxArea", "Other")]),
            Err("\"next\" is not a tax year".to_owned())
        );
        assert_eq!(
            approved_tax(&catalog, &[draft("taxYear", "2025"), draft("taxArea", "Holidays")]),
            Err("\"Holidays\" is not a tax area".to_owned())
        );
    }
}
