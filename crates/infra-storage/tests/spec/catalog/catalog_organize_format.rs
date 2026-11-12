//! The parts added in 2.1: contracts, cases, tax years and assignments, the owner, expected documents, the emergency
//! folder and the new inbox switches. All optional and left out while empty, so a 2.0 catalog is written unchanged.

use chrono::NaiveDate;
use dms_domain::cases::{CaseEntry, CaseNote, EntryDirection, FollowUp, FollowUpId, FollowUpSource};
use dms_domain::catalog::Catalog;
use dms_domain::contract::{
    Contract, ContractId, ContractKind, ContractPrice, ContractTerms, Notice, NoticeUnit, PricePoint,
};
use dms_domain::document::DocumentId;
use dms_domain::emergency::{EmergencyEntry, EmergencyFolder, EmergencyGroup, EmergencyItem, EmergencyItemId};
use dms_domain::expectation::{ExpectationMode, ExpectationOverride};
use dms_domain::field::{CurrencyCode, Decimal};
use dms_domain::money::{BillingPeriod, Money};
use dms_domain::settings::{CatalogSettings, InboxSettings, LibraryLayout, OwnerDetails};
use dms_domain::tax::{TaxArea, TaxAssignment, TaxChecklistItem, TaxItemId, TaxItemState, TaxYear};

use super::{Library, document};
use crate::support::{Applied, fixture};

const V2_1: &str = "catalog/catalog-2.1.yaml";

fn day(text: &str) -> NaiveDate {
    dms_domain::parse_iso_date(text).expect("date")
}

fn euros(amount: &str) -> Money {
    Money { amount: Decimal::parse(amount).expect("amount"), currency: CurrencyCode::new("EUR").expect("currency") }
}

fn id(value: &str) -> DocumentId {
    DocumentId::new(value).expect("id")
}

fn base() -> Catalog {
    let catalog = Catalog::empty(dms_domain::catalog::LibraryId::new("library-1").expect("id"));
    catalog
        .add_document(document("doc-a", "a.pdf", "first"))
        .applied()
        .add_document(document("doc-b", "b.pdf", "second"))
        .applied()
}

fn round_trip(catalog: &Catalog) -> Catalog {
    let library = Library::new();
    library.save(catalog);
    let found = library.found();
    library.save(&found);
    found
}

#[test]
fn a_contract_round_trips() {
    let price = ContractPrice {
        amount: Decimal::parse("29.90").expect("amount"),
        currency: euros("1").currency,
        period: BillingPeriod::Quarterly,
    };
    let mut contract =
        Contract::new(ContractId::new("c-1").expect("id"), "Internet: fibre", ContractKind::Telecom).expect("contract");
    contract.correspondent = Some("Example Telecom".to_owned());
    contract.price = Some(price.clone());
    contract.price_history = vec![PricePoint { valid_from: day("2026-01-01"), price, document_id: Some(id("doc-a")) }];
    contract.terms = ContractTerms {
        term_end: Some(day("2027-03-31")),
        renews_by_months: Some(1),
        notice: Some(Notice { count: 4, unit: NoticeUnit::Weeks }),
        cancellable_any_time: false,
    };
    contract.document_ids.extend([id("doc-b"), id("doc-a")]);
    contract.ended_on = Some(day("2027-03-31"));
    contract.note = Some("yes".to_owned());
    let catalog = base().save_contract(contract).applied();

    assert_eq!(round_trip(&catalog), catalog);
}

#[test]
fn a_case_round_trips() {
    let mut case = dms_domain::cases::Case::new(
        dms_domain::cases::CaseId::new("case-1").expect("id"),
        "Claim #1",
        day("2026-10-01"),
    )
    .expect("case");
    case.status = dms_domain::cases::CaseStatus::Done;
    case.closed_on = Some(day("2026-11-01"));
    case.remind_after_days = 7;
    case.entries =
        vec![CaseEntry { document_id: id("doc-a"), direction: EntryDirection::Received, date: day("2026-10-01") }];
    case.notes = vec![CaseNote { date: day("2026-10-02"), text: "Two\nlines".to_owned() }];
    case.follow_ups = vec![FollowUp {
        id: FollowUpId::new("f-1").expect("id"),
        date: day("2026-10-15"),
        text: "Reply".to_owned(),
        done: true,
        source: FollowUpSource::Ai { document_id: id("doc-a") },
    }];
    let catalog = base().save_case(case).applied();

    assert_eq!(round_trip(&catalog), catalog);
}

#[test]
fn a_tax_year_and_a_tax_assignment_round_trip() {
    let year = TaxYear {
        items: vec![TaxChecklistItem {
            id: TaxItemId::new("t-1").expect("id"),
            area: TaxArea::CapitalIncome,
            label: "Annual tax certificate".to_owned(),
            correspondent: Some("Example Bank".to_owned()),
            document_id: Some(id("doc-b")),
            state: TaxItemState::NotNeeded,
            note: None,
            added_by_user: false,
        }],
        sent_on: None,
        ..TaxYear::new(2025)
    };
    let catalog = base()
        .save_tax_year(year)
        .applied()
        .set_tax(&id("doc-a"), Some(TaxAssignment { year: 2025, area: TaxArea::Donations, deductible: None }))
        .applied()
        .set_tax(
            &id("doc-b"),
            Some(TaxAssignment { year: 2024, area: TaxArea::HealthCosts, deductible: Some(euros("0.50")) }),
        )
        .applied();

    assert_eq!(round_trip(&catalog), catalog);
}

#[test]
fn the_new_settings_round_trip() {
    let contract = Contract::new(ContractId::new("c-1").expect("id"), "Home insurance", ContractKind::Insurance)
        .expect("contract");
    let item = |index: usize, group, entry| EmergencyItem {
        id: EmergencyItemId::new(format!("e-{index}")).expect("id"),
        group,
        entry,
        included: index.is_multiple_of(2),
        recorded_hash: None,
    };
    let settings = CatalogSettings {
        owner: Some(OwnerDetails {
            name: "Alex Example".to_owned(),
            address_lines: vec!["Main Street 1".to_owned()],
            email: None,
            phone: Some("123".to_owned()),
        }),
        bills_tracked_since: Some(day("2026-11-01")),
        dismissed_contract_proposals: vec!["Example Gym".to_owned(), "null".to_owned()],
        expectations: vec![ExpectationOverride {
            correspondent: "Example Employer".to_owned(),
            category_id: None,
            mode: ExpectationMode::Off,
        }],
        emergency_folder: Some(EmergencyFolder {
            items: vec![
                item(0, EmergencyGroup::Accounts, EmergencyEntry::Document(id("doc-a"))),
                item(1, EmergencyGroup::Contracts, EmergencyEntry::Contract(contract.id.clone())),
                item(
                    2,
                    EmergencyGroup::Contacts,
                    EmergencyEntry::Contact {
                        name: "Sam".to_owned(),
                        role: None,
                        phone: None,
                        email: Some("sam@example.org".to_owned()),
                        note: Some("Has a key".to_owned()),
                    },
                ),
                item(3, EmergencyGroup::Other, EmergencyEntry::Note("on".to_owned())),
            ],
            last_refreshed: None,
            refresh_months: 6,
            intro_text: None,
        }),
        layout: LibraryLayout {
            inbox: InboxSettings { propose_splits: true, ..InboxSettings::default() },
            ..LibraryLayout::default()
        },
        ..CatalogSettings::default()
    };
    let catalog = base().save_contract(contract).applied().change_settings(settings).applied();

    assert_eq!(round_trip(&catalog), catalog);
}

#[test]
fn a_file_using_every_2_1_part_loads_and_is_written_back_to_the_byte_keeping_unknown_fields() {
    let library = Library::new();
    let original = fixture(V2_1);
    let catalog = library.load_text(&original);

    library.save(&catalog);

    assert_eq!(library.text(), original);
    assert_eq!(catalog.contracts().len(), 2);
    assert_eq!(catalog.cases().len(), 1);
    assert_eq!(catalog.tax_years()[&2025].items.len(), 2);
    assert_eq!(catalog.documents()[&id("doc-a")].tax.as_ref().map(|tax| tax.area), Some(TaxArea::HouseholdServices));
    let settings = catalog.settings();
    assert_eq!(settings.owner.as_ref().map(|owner| owner.address_lines.len()), Some(2));
    assert_eq!(settings.expectations.len(), 2);
    assert_eq!(settings.emergency_folder.as_ref().map(|folder| folder.items.len()), Some(4));
    let inbox = &settings.layout.inbox;
    assert!(inbox.propose_splits && inbox.convert_emails && inbox.keep_email_originals);
}

#[test]
fn references_an_older_version_left_behind_are_dropped_instead_of_failing() {
    // 2.0 removed doc-a and contract-1 but kept the 2.1 parts that pointed at them as unknown fields.
    let original = fixture(V2_1);
    let start = original.find("- id: doc-a").expect("doc-a");
    let end = original.find("- id: doc-b").expect("doc-b");
    let without_doc_a = format!("{}{}", &original[..start], &original[end..]);
    let without_contract = without_doc_a.replace("- id: contract-1\n", "- id: contract-removed\n");

    let catalog = Library::new().load_text(&without_contract);

    assert_eq!(catalog.documents().len(), 1);
    assert!(catalog.cases().values().all(|case| case.entries.iter().all(|entry| entry.document_id == id("doc-b"))));
    assert_eq!(catalog.tax_years()[&2025].items[0].document_id, None);
    let items = &catalog.settings().emergency_folder.as_ref().expect("folder").items;
    assert!(items.iter().all(|item| !matches!(item.entry, EmergencyEntry::Contract(_))), "contract-1 is gone");
}

#[test]
fn a_saved_search_with_a_tax_filter_round_trips() {
    use dms_domain::query::{DocumentQuery, SavedSearch, SavedSearchId};
    use dms_domain::tax::TaxFilter;
    use dms_domain::taxonomy::TagName;

    let search = |id: &str, tax| SavedSearch {
        id: SavedSearchId::new(id).expect("id"),
        name: id.to_owned(),
        query: DocumentQuery { tax: Some(tax), ..DocumentQuery::default() },
    };
    let settings = CatalogSettings {
        saved_searches: vec![
            search(
                "full",
                TaxFilter {
                    year: Some(2025),
                    area: Some(TaxArea::HouseholdServices),
                    tag_fallback: Some(TagName::of("Tax").expect("tag")),
                },
            ),
            search("area", TaxFilter { area: Some(TaxArea::Rental), ..TaxFilter::default() }),
        ],
        ..CatalogSettings::default()
    };
    let catalog = base().change_settings(settings).applied();

    assert_eq!(round_trip(&catalog), catalog);
}
