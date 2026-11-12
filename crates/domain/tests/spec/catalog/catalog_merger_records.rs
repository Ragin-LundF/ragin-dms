use dms_domain::catalog::{Catalog, CatalogMergeConflict, CatalogMerger};
use dms_domain::contract::Contract;
use dms_domain::emergency::EmergencyFolder;
use dms_domain::expectation::{ExpectationMode, ExpectationOverride};
use dms_domain::settings::{CatalogSettings, OwnerDetails};
use dms_domain::tax::TaxYear;

use crate::test_data::{self, Applied, date};

fn base() -> Catalog {
    test_data::catalog_with(&[test_data::document("letter", "letter.pdf")])
}

fn with_settings(catalog: &Catalog, change: impl FnOnce(&mut CatalogSettings)) -> Catalog {
    let mut settings = catalog.settings().clone();
    change(&mut settings);
    catalog.change_settings(settings).applied()
}

fn owner(name: &str) -> OwnerDetails {
    OwnerDetails { name: name.to_owned(), address_lines: vec!["1 Example Road".to_owned()], email: None, phone: None }
}

fn expectation(correspondent: &str, mode: ExpectationMode) -> ExpectationOverride {
    ExpectationOverride { correspondent: correspondent.to_owned(), category_id: None, mode }
}

#[test]
fn records_of_both_copies_are_kept_and_the_newer_copy_wins_a_reported_clash() {
    let shared = test_data::contract("c-shared", "Electricity");
    let older = base()
        .save_contract(shared.clone())
        .applied()
        .save_contract(test_data::contract("c-old", "Gym"))
        .applied()
        .save_case(test_data::case("case-old", "Objection"))
        .applied()
        .save_tax_year(TaxYear::new(2024))
        .applied();
    let newer = base()
        .save_contract(Contract { name: "Power".to_owned(), ..shared })
        .applied()
        .save_tax_year(TaxYear { sent_on: Some(date(2025, 5, 1)), ..TaxYear::new(2024) })
        .applied()
        .save_tax_year(TaxYear::new(2025))
        .applied()
        .save_case(test_data::case("case-new", "Claim"))
        .applied()
        .save_case(test_data::case("case-new", "Claim for damages"))
        .applied();
    assert!(newer.revision() > older.revision());

    let result = CatalogMerger::merge(&older, &newer).expect("same library");
    let merged = result.catalog;

    let clashes: Vec<String> = result
        .conflicts
        .iter()
        .map(|conflict| match conflict {
            CatalogMergeConflict::ContractDiffers { kept, discarded } => {
                format!("{} over {}", kept.name, discarded.name)
            }
            CatalogMergeConflict::TaxYearDiffers { kept, .. } => format!("tax year {}", kept.year),
            other => format!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(clashes, vec!["Power over Electricity".to_owned(), "tax year 2024".to_owned()]);
    assert_eq!(
        merged.contracts().values().map(|contract| contract.name.as_str()).collect::<Vec<_>>(),
        vec!["Power", "Gym"]
    );
    assert_eq!(merged.cases().len(), 2);
    assert_eq!(merged.tax_years()[&2024].sent_on, Some(date(2025, 5, 1)));
    assert_eq!(merged.tax_years().len(), 2);
}

#[test]
fn owner_and_emergency_folder_come_from_the_newer_copy_or_else_the_older_and_bills_are_tracked_since_the_earlier_day() {
    let older = with_settings(&base(), |settings| {
        settings.owner = Some(owner("Older Name"));
        settings.bills_tracked_since = Some(date(2026, 1, 1));
        settings.emergency_folder = Some(EmergencyFolder { refresh_months: 6, ..EmergencyFolder::default() });
    });
    let newer = with_settings(&with_settings(&base(), |settings| settings.owner = Some(owner("Newer Name"))), |_| {});

    let merged = CatalogMerger::merge(&older, &newer).expect("same library").catalog;
    let settings = merged.settings();

    assert_eq!(settings.owner, Some(owner("Newer Name")));
    assert_eq!(settings.bills_tracked_since, Some(date(2026, 1, 1)));
    assert_eq!(settings.emergency_folder.as_ref().map(|folder| folder.refresh_months), Some(6));
}

#[test]
fn different_owner_details_and_emergency_folders_are_reported_and_the_earlier_tracking_day_wins() {
    let older = with_settings(&base(), |settings| {
        settings.owner = Some(owner("Older Name"));
        settings.bills_tracked_since = Some(date(2026, 1, 1));
        settings.emergency_folder = Some(EmergencyFolder { refresh_months: 6, ..EmergencyFolder::default() });
    });
    let newer = with_settings(&with_settings(&base(), |_| {}), |settings| {
        settings.owner = Some(owner("Newer Name"));
        settings.bills_tracked_since = Some(date(2026, 3, 1));
        settings.emergency_folder = Some(EmergencyFolder::default());
    });

    let result = CatalogMerger::merge(&older, &newer).expect("same library");

    assert_eq!(result.catalog.settings().owner, Some(owner("Newer Name")));
    assert_eq!(result.catalog.settings().bills_tracked_since, Some(date(2026, 1, 1)));
    assert_eq!(result.catalog.settings().emergency_folder, Some(EmergencyFolder::default()));
    assert!(matches!(
        result.conflicts.as_slice(),
        [CatalogMergeConflict::OwnerDiffers { .. }, CatalogMergeConflict::EmergencyFolderDiffers { .. }]
    ));
}

#[test]
fn dismissed_proposals_are_united_ignoring_case_and_expectations_by_sender_with_the_newer_winning() {
    let older = with_settings(&base(), |settings| {
        settings.dismissed_contract_proposals = vec!["Water Works".to_owned(), "Gym Club".to_owned()];
        settings.expectations =
            vec![expectation("Employer", ExpectationMode::Keep), expectation("Insurer", ExpectationMode::Off)];
    });
    let newer = with_settings(
        &with_settings(&base(), |settings| {
            settings.dismissed_contract_proposals = vec!["water works".to_owned()];
            settings.expectations = vec![expectation("EMPLOYER", ExpectationMode::Off)];
        }),
        |_| {},
    );

    let merged = CatalogMerger::merge(&older, &newer).expect("same library").catalog;

    assert_eq!(merged.settings().dismissed_contract_proposals, vec!["water works".to_owned(), "Gym Club".to_owned()]);
    assert_eq!(
        merged.settings().expectations,
        vec![expectation("EMPLOYER", ExpectationMode::Off), expectation("Insurer", ExpectationMode::Off)]
    );
}

#[test]
fn a_record_of_a_document_dropped_by_a_path_clash_loses_that_reference() {
    let empty = Catalog::empty(test_data::library_id());
    let kept = test_data::document("kept", "same.pdf");
    let discarded = test_data::document("discarded", "same.pdf");
    let mut contract = test_data::contract("c-1", "Electricity");
    contract.document_ids.insert(discarded.id.clone());
    let older = empty.add_document(discarded).applied().save_contract(contract).applied();
    let newer = empty
        .add_document(kept)
        .applied()
        .set_note(&test_data::document_id("kept"), Some("x"))
        .applied()
        .set_note(&test_data::document_id("kept"), Some("y"))
        .applied();

    let merged = CatalogMerger::merge(&older, &newer).expect("same library").catalog;

    assert_eq!(merged.contracts()[0].document_ids.len(), 0);
}
