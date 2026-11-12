use dms_application::common::OperationOutcome;
use dms_domain::catalog::CatalogRejection;
use dms_domain::tax::{TaxArea, TaxAssignment, TaxItemState, TaxSums};

use super::tax_kit::{YEAR, add_tax_document, assignment, item, stored_year, tax_library};
use crate::support::test_data::{date, document_id, done};

#[test]
fn opening_a_year_lists_last_years_senders_and_writes_only_when_the_list_changed() {
    let library = tax_library();
    add_tax_document(
        &library,
        "a.pdf",
        "Chimney Sweep",
        date(2022, 11, 3),
        Some(assignment(2022, TaxArea::HouseholdServices)),
        None,
    );

    let first = done(library.open_tax_year().run(YEAR));
    let again = done(library.open_tax_year().run(YEAR));

    let year = stored_year(&library, YEAR);
    assert_eq!(item(&year, "Chimney Sweep").state, TaxItemState::Missing, "usually came on 3 Nov");
    assert_eq!(item(&year, "Lohnsteuerbescheinigung").state, TaxItemState::Missing);
    assert_eq!(again.revision(), first.revision(), "an unchanged checklist is not written again");
}

#[test]
fn assigning_a_document_ticks_its_item_and_updates_the_sum() {
    let library = tax_library();
    add_tax_document(
        &library,
        "old.pdf",
        "Chimney Sweep",
        date(2022, 11, 3),
        Some(assignment(2022, TaxArea::HouseholdServices)),
        None,
    );
    done(library.open_tax_year().run(YEAR));
    let bill = add_tax_document(&library, "bill.pdf", "chimney sweep", date(2023, 11, 2), None, Some("96.00"));

    done(library.assign_tax().run(
        std::slice::from_ref(&bill.id),
        Some(&assignment(YEAR, TaxArea::HouseholdServices)),
        true,
    ));

    let catalog = library.catalog();
    let year = stored_year(&library, YEAR);
    assert_eq!(item(&year, "Chimney Sweep").state, TaxItemState::Found);
    assert_eq!(item(&year, "Chimney Sweep").document_id, Some(bill.id));
    let sums = TaxSums::of(&catalog, YEAR);
    let amounts = sums[&TaxArea::HouseholdServices].amounts();
    assert_eq!(amounts[0].amount, dms_domain::field::Decimal::parse("96.00").expect("amount"));
}

#[test]
fn the_bulk_bar_keeps_each_deductible_part_and_none_removes_the_assignment() {
    let library = tax_library();
    let labour = dms_domain::money::Money {
        amount: dms_domain::field::Decimal::parse("684.50").expect("amount"),
        currency: dms_domain::field::CurrencyCode::new("EUR").expect("currency"),
    };
    let plumber = add_tax_document(&library, "plumber.pdf", "Plumber", date(2023, 4, 1), None, None);
    let other = add_tax_document(&library, "other.pdf", "Glazier", date(2023, 5, 1), None, None);
    let with_labour =
        TaxAssignment { deductible: Some(labour.clone()), ..assignment(2022, TaxArea::HouseholdServices) };
    done(library.assign_tax().run(std::slice::from_ref(&plumber.id), Some(&with_labour), false));

    let ids = [plumber.id.clone(), other.id.clone()];
    done(library.assign_tax().run(&ids, Some(&assignment(YEAR, TaxArea::HouseholdServices)), true));

    let catalog = library.catalog();
    assert_eq!(
        catalog.documents()[&plumber.id].tax,
        Some(TaxAssignment { deductible: Some(labour), ..assignment(YEAR, TaxArea::HouseholdServices) })
    );
    assert_eq!(catalog.documents()[&other.id].tax, Some(assignment(YEAR, TaxArea::HouseholdServices)));

    done(library.assign_tax().run(&ids, None, true));
    assert!(library.catalog().documents().values().all(|document| document.tax.is_none()));
}

#[test]
fn an_unknown_document_or_an_invalid_year_changes_nothing() {
    let library = tax_library();
    let known = add_tax_document(&library, "a.pdf", "Shop", date(2023, 4, 1), None, None);
    let before = library.catalog();

    let unknown = library.assign_tax().run(
        &[known.id.clone(), document_id("gone")],
        Some(&assignment(YEAR, TaxArea::Other)),
        true,
    );
    let invalid = library.assign_tax().run(&[known.id], Some(&assignment(12, TaxArea::Other)), true);

    assert_eq!(unknown, OperationOutcome::Rejected(CatalogRejection::DocumentNotFound));
    assert!(matches!(invalid, OperationOutcome::InvalidInput(_)));
    assert_eq!(library.catalog(), before);
}
