use dms_domain::cases::{CaseEntry, CaseId, EntryDirection};
use dms_domain::catalog::{Catalog, CatalogRecords, CatalogRejection, CatalogUpdate};
use dms_domain::contract::{ContractId, ContractPrice, PricePoint};
use dms_domain::document::Document;
use dms_domain::emergency::{EmergencyEntry, EmergencyFolder, EmergencyGroup, EmergencyItem, EmergencyItemId};
use dms_domain::money::BillingPeriod;
use dms_domain::settings::CatalogSettings;
use dms_domain::tax::{TaxArea, TaxAssignment, TaxChecklistItem, TaxItemId, TaxItemState, TaxYear};
use indexmap::IndexMap;

use crate::test_data::{self, Applied, date, decimal, euro, euros};

struct Fixture {
    letter: Document,
    bill: Document,
    base: Catalog,
}

fn fixture() -> Fixture {
    let letter = test_data::document("letter", "letter.pdf");
    let bill = test_data::document("bill", "bill.pdf");
    let base = test_data::catalog_with(&[letter.clone(), bill.clone()]);
    Fixture { letter, bill, base }
}

fn emergency(entries: Vec<EmergencyEntry>) -> EmergencyFolder {
    let items = entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| EmergencyItem {
            id: EmergencyItemId::new(format!("item-{index}")).expect("valid id"),
            group: EmergencyGroup::Other,
            entry,
            included: true,
            recorded_hash: None,
        })
        .collect();
    EmergencyFolder { items, ..EmergencyFolder::default() }
}

fn tax_year_with(f: &Fixture) -> TaxYear {
    TaxYear {
        items: vec![TaxChecklistItem {
            id: TaxItemId::new("item-1").expect("valid id"),
            area: TaxArea::HouseholdServices,
            label: "Chimney sweep".to_owned(),
            correspondent: None,
            document_id: Some(f.bill.id.clone()),
            state: TaxItemState::Found,
            note: None,
            added_by_user: false,
        }],
        ..TaxYear::new(2025)
    }
}

#[test]
fn records_are_saved_replaced_in_place_and_deleted_each_with_a_new_revision() {
    let f = fixture();
    let first = test_data::contract("c-1", "Electricity");
    let second = test_data::contract("c-2", "Gym");

    let saved = f.base.save_contract(first.clone()).applied().save_contract(second.clone()).applied();
    let renamed = saved.save_contract(dms_domain::contract::Contract { name: "Power".to_owned(), ..first }).applied();
    let deleted = renamed.delete_contract(&second.id).applied();

    assert_eq!(saved.revision().value(), f.base.revision().value() + 2);
    assert_eq!(renamed.contracts().keys().collect::<Vec<_>>(), vec![&ContractId::new("c-1").expect("id"), &second.id]);
    assert_eq!(renamed.contracts()[0].name, "Power");
    assert_eq!(deleted.contracts().len(), 1);
    assert_eq!(f.base.delete_contract(&second.id), CatalogUpdate::Rejected(CatalogRejection::ContractNotFound));
    let case = test_data::case("case-1", "Objection");
    assert_eq!(f.base.save_case(case.clone()).applied().delete_case(&case.id).applied().cases().len(), 0);
    assert_eq!(f.base.delete_case(&case.id), CatalogUpdate::Rejected(CatalogRejection::CaseNotFound));
    assert_eq!(f.base.save_tax_year(tax_year_with(&f)).applied().tax_years()[&2025], tax_year_with(&f));
}

#[test]
fn a_record_that_refers_to_an_unknown_document_is_rejected_and_an_invalid_one_is_an_error() {
    let f = fixture();
    let mut contract = test_data::contract("c-1", "Electricity");
    contract.document_ids.insert(test_data::document_id("gone"));
    let mut case = test_data::case("case-1", "Objection");
    case.entries.push(CaseEntry {
        document_id: test_data::document_id("gone"),
        direction: EntryDirection::Sent,
        date: date(2026, 1, 5),
    });

    assert_eq!(f.base.save_contract(contract.clone()), Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound)));
    assert_eq!(f.base.save_case(case), Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound)));
    assert!(
        f.base
            .save_contract(dms_domain::contract::Contract { name: " ".to_owned(), ..test_data::contract("c-2", "x") })
            .is_err()
    );
    assert!(f.base.save_tax_year(TaxYear::new(99)).is_err());
}

#[test]
fn a_tax_assignment_is_set_and_cleared_on_the_document() {
    let f = fixture();
    let tax = TaxAssignment { year: 2025, area: TaxArea::HouseholdServices, deductible: Some(euros("684.50")) };

    let assigned = f.base.set_tax(&f.bill.id, Some(tax.clone())).applied();
    let cleared = assigned.set_tax(&f.bill.id, None).applied();

    assert_eq!(assigned.documents()[&f.bill.id].tax, Some(tax));
    assert_eq!(cleared.documents()[&f.bill.id].tax, None);
    assert!(
        f.base.set_tax(&f.bill.id, Some(TaxAssignment { year: 12, area: TaxArea::Other, deductible: None })).is_err()
    );
    assert_eq!(
        f.base.set_tax(&test_data::document_id("gone"), None),
        Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound))
    );
}

#[test]
fn removing_a_document_removes_it_from_every_record_and_the_emergency_folder() {
    let f = fixture();
    let mut contract = test_data::contract("c-1", "Electricity");
    contract.document_ids.extend([f.bill.id.clone(), f.letter.id.clone()]);
    let price = ContractPrice { amount: decimal("80"), currency: euro(), period: BillingPeriod::Monthly };
    contract.price_history.push(PricePoint {
        valid_from: date(2026, 1, 1),
        price,
        document_id: Some(f.bill.id.clone()),
    });
    let mut case = test_data::case("case-1", "Objection");
    case.entries.push(CaseEntry {
        document_id: f.bill.id.clone(),
        direction: EntryDirection::Received,
        date: date(2026, 1, 5),
    });
    let folder = emergency(vec![
        EmergencyEntry::Document(f.bill.id.clone()),
        EmergencyEntry::Note("Keys at the neighbour".to_owned()),
    ]);
    let catalog = f
        .base
        .save_contract(contract)
        .applied()
        .save_case(case)
        .applied()
        .save_tax_year(tax_year_with(&f))
        .applied()
        .change_settings(CatalogSettings { emergency_folder: Some(folder), ..CatalogSettings::default() })
        .applied();

    let removed = catalog.remove_document(&f.bill.id).applied();

    let contract = &removed.contracts()[0];
    assert_eq!(contract.document_ids.iter().collect::<Vec<_>>(), vec![&f.letter.id]);
    assert_eq!(contract.price_history[0].document_id, None);
    assert_eq!(removed.cases()[&CaseId::new("case-1").expect("id")].entries, Vec::new());
    let item = &removed.tax_years()[&2025].items[0];
    assert_eq!((item.document_id.as_ref(), item.state), (None, TaxItemState::Found));
    let items = &removed.settings().emergency_folder.as_ref().expect("folder").items;
    assert_eq!(
        items.iter().map(|item| &item.entry).collect::<Vec<_>>(),
        vec![&EmergencyEntry::Note("Keys at the neighbour".to_owned())]
    );
}

#[test]
fn deleting_a_contract_removes_its_emergency_item() {
    let f = fixture();
    let contract = test_data::contract("c-1", "Electricity");
    let catalog = f.base.save_contract(contract.clone()).applied();
    let folder = emergency(vec![EmergencyEntry::Contract(contract.id.clone())]);
    let catalog = catalog
        .change_settings(CatalogSettings { emergency_folder: Some(folder), ..CatalogSettings::default() })
        .applied();

    let deleted = catalog.delete_contract(&contract.id).applied();

    assert_eq!(deleted.settings().emergency_folder.as_ref().map(|folder| folder.items.len()), Some(0));
}

#[test]
fn with_records_checks_keys_document_references_and_emergency_contracts() {
    let f = fixture();
    let contract = test_data::contract("c-1", "Electricity");
    let mismatched = CatalogRecords {
        contracts: IndexMap::from([(ContractId::new("other").expect("id"), contract.clone())]),
        ..CatalogRecords::default()
    };
    let mut dangling = contract.clone();
    dangling.document_ids.insert(test_data::document_id("gone"));
    let dangling =
        CatalogRecords { contracts: IndexMap::from([(dangling.id.clone(), dangling)]), ..CatalogRecords::default() };
    let unknown_contract = f
        .base
        .change_settings(CatalogSettings {
            emergency_folder: Some(emergency(vec![EmergencyEntry::Document(f.letter.id.clone())])),
            ..CatalogSettings::default()
        })
        .applied();
    let valid = CatalogRecords {
        contracts: IndexMap::from([(contract.id.clone(), contract.clone())]),
        ..CatalogRecords::default()
    };

    assert!(f.base.clone().with_records(mismatched).is_err());
    assert!(f.base.clone().with_records(dangling).is_err());
    let with_records = unknown_contract.clone().with_records(valid.clone()).expect("valid records");
    assert_eq!((with_records.revision(), with_records.contracts().len()), (unknown_contract.revision(), 1));
    let settings = CatalogSettings {
        emergency_folder: Some(emergency(vec![EmergencyEntry::Contract(ContractId::new("gone").expect("id"))])),
        ..CatalogSettings::default()
    };
    assert!(f.base.change_settings(settings).is_err(), "an emergency item of an unknown contract");
}
