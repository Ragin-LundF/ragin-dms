use dms_domain::catalog::Catalog;
use dms_domain::contract::{ContractPrice, PricePoint};
use dms_domain::document::ContentHash;
use dms_domain::emergency::{
    EmergencyChanges, EmergencyEntry, EmergencyFolder, EmergencyGroup, EmergencyItem, EmergencyItemId, EmergencyPlanner,
};
use dms_domain::money::BillingPeriod;
use dms_domain::settings::CatalogSettings;
use dms_domain::task::OpenTask;

use crate::test_data::{Applied, catalog_with, contract, date, decimal, document, euro, hash};

fn item(id: &str, entry: EmergencyEntry, recorded_hash: Option<ContentHash>) -> EmergencyItem {
    EmergencyItem {
        id: EmergencyItemId::new(id).expect("id"),
        group: EmergencyGroup::Other,
        entry,
        included: true,
        recorded_hash,
    }
}

fn with_folder(catalog: &Catalog, folder: EmergencyFolder) -> Catalog {
    catalog.change_settings(CatalogSettings { emergency_folder: Some(folder), ..catalog.settings().clone() }).applied()
}

fn note(id: &str) -> EmergencyItem {
    item(id, EmergencyEntry::Note("Keys are with the neighbour".to_owned()), None)
}

#[test]
fn a_document_whose_content_differs_from_the_recorded_one_is_changed() {
    let unchanged = document("d-1", "a.pdf");
    let edited = document("d-2", "b.pdf");
    let catalog = catalog_with(&[unchanged.clone(), edited.clone()]);
    let folder = EmergencyFolder {
        items: vec![
            item("e-1", EmergencyEntry::Document(unchanged.id.clone()), Some(unchanged.content_hash.clone())),
            item("e-2", EmergencyEntry::Document(edited.id.clone()), Some(hash(1))),
            item("e-3", EmergencyEntry::Document(unchanged.id.clone()), None),
            note("e-4"),
        ],
        last_refreshed: Some(date(2026, 1, 10)),
        ..EmergencyFolder::default()
    };
    let catalog = with_folder(&catalog, folder);
    let folder = catalog.settings().emergency_folder.as_ref().expect("folder");

    let changed: Vec<String> = EmergencyChanges::of(folder, &catalog).iter().map(ToString::to_string).collect();

    assert_eq!(changed, vec!["e-2".to_owned(), "e-3".to_owned()]);
}

#[test]
fn a_folder_that_was_never_refreshed_has_no_changes() {
    let edited = document("d-2", "b.pdf");
    let catalog = catalog_with(std::slice::from_ref(&edited));
    let folder =
        EmergencyFolder { items: vec![item("e-1", EmergencyEntry::Document(edited.id), None)], ..Default::default() };
    let catalog = with_folder(&catalog, folder.clone());

    assert_eq!(EmergencyChanges::of(&folder, &catalog).len(), 0);
}

#[test]
fn a_contract_with_a_new_price_or_an_end_after_the_refresh_is_changed() {
    let price = |from| PricePoint {
        valid_from: from,
        price: ContractPrice { amount: decimal("10.00"), currency: euro(), period: BillingPeriod::Monthly },
        document_id: None,
    };
    let older =
        dms_domain::contract::Contract { price_history: vec![price(date(2025, 1, 1))], ..contract("c-1", "Gym") };
    let raised =
        dms_domain::contract::Contract { price_history: vec![price(date(2026, 3, 1))], ..contract("c-2", "Power") };
    let ended = dms_domain::contract::Contract { ended_on: Some(date(2026, 2, 1)), ..contract("c-3", "Phone") };
    let catalog = [older, raised, ended]
        .into_iter()
        .fold(catalog_with(&[]), |catalog, contract| catalog.save_contract(contract).applied());
    let folder = EmergencyFolder {
        items: ["c-1", "c-2", "c-3"]
            .iter()
            .map(|id| {
                let contract_id = dms_domain::contract::ContractId::new(*id).expect("id");
                item(&format!("e-{id}"), EmergencyEntry::Contract(contract_id), None)
            })
            .collect(),
        last_refreshed: Some(date(2026, 1, 10)),
        ..EmergencyFolder::default()
    };
    let catalog = with_folder(&catalog, folder.clone());

    let changed: Vec<String> = EmergencyChanges::of(&folder, &catalog).iter().map(ToString::to_string).collect();

    assert_eq!(changed, vec!["e-c-2".to_owned(), "e-c-3".to_owned()]);
}

#[test]
fn the_refresh_task_appears_fourteen_days_before_it_is_due() {
    let folder = EmergencyFolder {
        items: vec![note("e-1")],
        last_refreshed: Some(date(2025, 11, 14)),
        refresh_months: 12,
        intro_text: None,
    };
    let catalog = with_folder(&catalog_with(&[]), folder);

    assert_eq!(EmergencyPlanner::due_tasks(&catalog, date(2026, 10, 30)), Vec::<OpenTask>::new());
    assert_eq!(
        EmergencyPlanner::due_tasks(&catalog, date(2026, 10, 31)),
        vec![OpenTask::EmergencyFolderRefresh { due: date(2026, 11, 14) }]
    );
    assert_eq!(
        EmergencyPlanner::due_tasks(&catalog, date(2027, 1, 5)),
        vec![OpenTask::EmergencyFolderRefresh { due: date(2026, 11, 14) }]
    );
}

#[test]
fn an_empty_or_never_refreshed_folder_asks_for_no_refresh() {
    let never = with_folder(&catalog_with(&[]), EmergencyFolder { items: vec![note("e-1")], ..Default::default() });
    let empty = with_folder(
        &catalog_with(&[]),
        EmergencyFolder { last_refreshed: Some(date(2020, 1, 1)), ..EmergencyFolder::default() },
    );

    assert_eq!(EmergencyPlanner::due_tasks(&never, date(2030, 1, 1)), Vec::<OpenTask>::new());
    assert_eq!(EmergencyPlanner::due_tasks(&empty, date(2030, 1, 1)), Vec::<OpenTask>::new());
    assert_eq!(EmergencyPlanner::due_tasks(&catalog_with(&[]), date(2030, 1, 1)), Vec::<OpenTask>::new());
}
