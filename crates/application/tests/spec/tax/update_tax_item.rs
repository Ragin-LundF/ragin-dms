use dms_application::common::OperationOutcome;
use dms_application::tax::TaxItemChange;
use dms_domain::tax::{TaxArea, TaxItemId, TaxItemState};

use super::tax_kit::{YEAR, add_tax_document, assignment, item, stored_year, tax_library};
use crate::support::test_data::{date, done};

#[test]
fn a_users_own_item_can_be_added_renamed_and_marked_not_needed_and_needed_again() {
    let library = tax_library();
    let change = |change: TaxItemChange| done(library.update_tax_item().run(YEAR, &change));

    change(TaxItemChange::Add { area: TaxArea::WorkExpenses, label: " Laptop ".to_owned(), correspondent: None });
    let laptop = item(&stored_year(&library, YEAR), "Laptop").clone();
    change(TaxItemChange::Rename { item_id: laptop.id.clone(), label: "Work laptop".to_owned() });
    change(TaxItemChange::SetNotNeeded { item_id: laptop.id.clone(), not_needed: true });
    done(library.open_tax_year().run(YEAR));

    let year = stored_year(&library, YEAR);
    let renamed = item(&year, "Work laptop");
    assert!(renamed.added_by_user);
    assert_eq!(renamed.state, TaxItemState::NotNeeded, "a rebuild keeps it");

    change(TaxItemChange::SetNotNeeded { item_id: laptop.id, not_needed: false });
    assert_eq!(item(&stored_year(&library, YEAR), "Work laptop").state, TaxItemState::Missing);
}

#[test]
fn i_have_it_links_the_document_and_assigns_it_to_the_year_and_area() {
    let library = tax_library();
    let receipt = add_tax_document(
        &library,
        "receipt.pdf",
        "Some Charity",
        date(2023, 12, 20),
        Some(assignment(2021, TaxArea::Other)),
        None,
    );
    done(library.update_tax_item().run(
        YEAR,
        &TaxItemChange::Add { area: TaxArea::Donations, label: "Donation receipt".to_owned(), correspondent: None },
    ));
    let wanted = item(&stored_year(&library, YEAR), "Donation receipt").id.clone();

    done(
        library
            .update_tax_item()
            .run(YEAR, &TaxItemChange::LinkDocument { item_id: wanted, document_id: receipt.id.clone() }),
    );

    let year = stored_year(&library, YEAR);
    assert_eq!(item(&year, "Donation receipt").state, TaxItemState::Found);
    assert_eq!(item(&year, "Donation receipt").document_id, Some(receipt.id.clone()));
    assert_eq!(library.catalog().documents()[&receipt.id].tax, Some(assignment(YEAR, TaxArea::Donations)));
}

#[test]
fn an_unknown_item_is_not_found_and_a_blank_label_is_refused() {
    let library = tax_library();
    done(library.open_tax_year().run(YEAR));
    let before = library.catalog();
    let gone = TaxItemId::new("gone").expect("id");
    let standard = item(&stored_year(&library, YEAR), "Lohnsteuerbescheinigung").id.clone();

    let unknown = library.update_tax_item().run(YEAR, &TaxItemChange::SetNotNeeded { item_id: gone, not_needed: true });
    let blank =
        library.update_tax_item().run(YEAR, &TaxItemChange::Rename { item_id: standard, label: "  ".to_owned() });

    assert_eq!(unknown, OperationOutcome::NotFound);
    assert!(matches!(blank, OperationOutcome::InvalidInput(_)));
    assert_eq!(library.catalog(), before);
}
