use chrono::NaiveDate;
use dms_application::catalog::CatalogEdit;
use dms_application::document::DetailEdit;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::field::{FieldPresets, FieldValue};
use dms_domain::settings::CatalogSettings;
use dms_domain::tax::{TaxArea, TaxAssignment, TaxChecklistItem, TaxYear};
use indexmap::IndexMap;

use crate::support::TestLibrary;
use crate::support::test_data::done;

/// The year the tests build checklists for; the fixed clock stands on 2024-03-01.
pub const YEAR: i32 = 2023;

pub fn assignment(year: i32, area: TaxArea) -> TaxAssignment {
    TaxAssignment { year, area, deductible: None }
}

/// An open library with the amount field.
pub fn tax_library() -> TestLibrary {
    let library = TestLibrary::new();
    library.open();
    done(library.change_settings().run("fields", |_| CatalogSettings {
        field_definitions: vec![FieldPresets::amount()],
        ..CatalogSettings::default()
    }));
    library
}

/// Adds a document from `sender`, dated `on`, assigned to `tax` and with an amount, if given.
pub fn add_tax_document(
    library: &TestLibrary,
    path: &str,
    sender: &str,
    on: NaiveDate,
    tax: Option<TaxAssignment>,
    amount: Option<&str>,
) -> Document {
    let document = library.add(path, path);
    let metadata = DocumentMetadata { title: None, document_date: Some(on), correspondent: Some(sender.to_owned()) };
    done(library.edit_catalog().run(&CatalogEdit::UpdateMetadata { document_id: document.id.clone(), metadata }));
    if let Some(amount) = amount {
        let value = FieldValue::Money {
            amount: dms_domain::field::Decimal::parse(amount).expect("amount"),
            currency: dms_domain::field::CurrencyCode::new("EUR").expect("currency"),
        };
        let fields = IndexMap::from([(FieldPresets::amount().key, value)]);
        done(library.edit_document_details().run(&DetailEdit::SetFields { document_id: document.id.clone(), fields }));
    }
    if let Some(tax) = tax {
        done(library.assign_tax().run(std::slice::from_ref(&document.id), Some(&tax), false));
    }
    library.catalog().documents()[&document.id].clone()
}

pub fn stored_year(library: &TestLibrary, year: i32) -> TaxYear {
    library.catalog().tax_years().get(&year).cloned().expect("the year is stored")
}

pub fn item<'a>(year: &'a TaxYear, label: &str) -> &'a TaxChecklistItem {
    year.items.iter().find(|item| item.label == label).unwrap_or_else(|| panic!("no item {label}"))
}
