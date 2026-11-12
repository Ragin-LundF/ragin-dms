//! The tax row of the quick look and the documents "I have it" offers.

use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::tax::{TaxItemId, TaxSums};
use dms_domain::{iso_date, text};

use super::amounts::{area_dto, area_options, default_year, money_display, money_text};
use crate::features::tax::dto::{TaxDocumentChoiceDto, TaxDocumentDto};

const MAX_CHOICES: usize = 200;

pub fn present_document(document: &Document, today: NaiveDate) -> TaxDocumentDto {
    let tax = document.tax.as_ref();
    let text = tax.map(|tax| {
        let amount = TaxSums::deductible(document).map(|money| format!(" · {}", money_display(&money)));
        format!("{} · {}{}", tax.year, tax.area.label(), amount.unwrap_or_default())
    });
    TaxDocumentDto {
        text,
        year: tax.map(|tax| tax.year),
        area: tax.map(|tax| area_dto(tax.area)),
        deductible: tax.and_then(|tax| tax.deductible.as_ref()).map(money_text),
        default_year: default_year(today),
        area_options: area_options(),
    }
}

/// The documents for an item of `year`: from the item's sender first, then those dated in the year or the one
/// after it, then all others, each group newest first; at most [`MAX_CHOICES`].
pub fn present_choices(catalog: &Catalog, year: i32, item_id: &TaxItemId) -> Vec<TaxDocumentChoiceDto> {
    let sender = catalog
        .tax_years()
        .get(&year)
        .and_then(|tax_year| tax_year.items.iter().find(|item| &item.id == item_id))
        .and_then(|item| item.correspondent.as_deref())
        .map(|name| text::lowercase(text::trim(name)));
    let rank = |document: &Document| {
        let from_sender = sender.as_ref().is_some_and(|wanted| {
            document.metadata.correspondent.as_deref().map(|name| text::lowercase(text::trim(name))).as_ref()
                == Some(wanted)
        });
        let dated_near = document.metadata.document_date.is_some_and(|date| {
            use chrono::Datelike;
            date.year() == year || date.year() == year + 1
        });
        (!from_sender, !dated_near, std::cmp::Reverse(document.metadata.document_date))
    };
    let mut documents: Vec<&Document> = catalog.documents().values().collect();
    documents.sort_by_key(|document| rank(document));
    documents
        .into_iter()
        .take(MAX_CHOICES)
        .map(|document| TaxDocumentChoiceDto {
            id: document.id.value().to_owned(),
            name: document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned()),
            correspondent: document.metadata.correspondent.clone(),
            date: document.metadata.document_date.map(iso_date),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use dms_domain::tax::{TaxArea, TaxChecklistItem, TaxItemState, TaxYear};

    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, date};

    #[test]
    fn the_row_names_year_area_and_the_deductible_part() {
        let catalog = TestCatalog::new()
            .document("plumber.pdf", |document| {
                document.money("amount", "1200.00").tax(2025, TaxArea::HouseholdServices, Some("684.50"));
            })
            .document("plain.pdf", |_| {})
            .build();
        let rows: Vec<TaxDocumentDto> =
            catalog.documents().values().map(|document| present_document(document, date(2026, 9, 1))).collect();

        assert_eq!(rows[0].text.as_deref(), Some("2025 · Household services and craftsmen · € 684.50"));
        assert_eq!(rows[0].deductible.as_deref(), Some("684.50 EUR"));
        assert_eq!((rows[1].text.as_deref(), rows[1].default_year), (None, 2026));
    }

    #[test]
    fn choices_start_with_the_items_sender_then_documents_of_the_year() {
        let catalog = TestCatalog::new()
            .document("old.pdf", |document| {
                document.correspondent("Shop").dated(date(2020, 1, 1));
            })
            .document("near.pdf", |document| {
                document.correspondent("Shop").dated(date(2026, 2, 1));
            })
            .document("sender.pdf", |document| {
                document.correspondent("sea rescue").dated(date(2019, 1, 1));
            })
            .build();
        let mut year = TaxYear::new(2025);
        year.items.push(TaxChecklistItem {
            id: TaxItemId::new("t-1").expect("id"),
            area: TaxArea::Donations,
            label: "Sea Rescue".to_owned(),
            correspondent: Some("Sea Rescue".to_owned()),
            document_id: None,
            state: TaxItemState::Missing,
            note: None,
            added_by_user: false,
        });
        let catalog = match catalog.save_tax_year(year).expect("valid") {
            dms_domain::catalog::CatalogUpdate::Applied(catalog) => catalog,
            dms_domain::catalog::CatalogUpdate::Rejected(reason) => panic!("{reason:?}"),
        };

        let names: Vec<String> = present_choices(&catalog, 2025, &TaxItemId::new("t-1").expect("id"))
            .into_iter()
            .map(|choice| choice.name)
            .collect();

        assert_eq!(names, ["sender.pdf", "near.pdf", "old.pdf"]);
    }
}
