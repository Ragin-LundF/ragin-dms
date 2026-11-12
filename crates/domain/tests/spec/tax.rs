use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::field::{FieldPresets, FieldValue};
use dms_domain::money::Money;
use dms_domain::query::{DocumentQuery, QueryContext};
use dms_domain::tax::{
    TaxArea, TaxAssignment, TaxChecklistBuilder, TaxChecklistItem, TaxFilter, TaxItemId, TaxItemState, TaxSums, TaxYear,
};

use crate::test_data::{self, Applied, date, euros, tag};

struct Paper<'a> {
    id: &'a str,
    sender: Option<&'a str>,
    title: Option<&'a str>,
    on: NaiveDate,
    tax: Option<(i32, TaxArea)>,
}

fn paper<'a>(id: &'a str, sender: &'a str, on: NaiveDate, tax: Option<(i32, TaxArea)>) -> Paper<'a> {
    Paper { id, sender: Some(sender), title: None, on, tax }
}

fn document_of(paper: &Paper<'_>) -> Document {
    let mut document = test_data::document(paper.id, &format!("tax/{}.pdf", paper.id));
    document.metadata.correspondent = paper.sender.map(str::to_owned);
    document.metadata.title = paper.title.map(str::to_owned);
    document.metadata.document_date = Some(paper.on);
    document.tax = paper.tax.map(|(year, area)| TaxAssignment { year, area, deductible: None });
    document
}

fn catalog_of(papers: &[Paper<'_>]) -> Catalog {
    test_data::catalog_with(&papers.iter().map(document_of).collect::<Vec<_>>())
}

fn ids() -> impl FnMut() -> TaxItemId {
    let mut next = 0;
    move || {
        next += 1;
        TaxItemId::new(format!("item-{next}")).expect("valid id")
    }
}

fn build(catalog: &Catalog, year: i32, today: NaiveDate) -> TaxYear {
    TaxChecklistBuilder::build(catalog, year, today, &mut ids())
}

fn item<'a>(year: &'a TaxYear, label: &str) -> &'a TaxChecklistItem {
    year.items.iter().find(|item| item.label == label).unwrap_or_else(|| panic!("no item {label}"))
}

fn with_year(catalog: &Catalog, year: TaxYear) -> Catalog {
    catalog.save_tax_year(year).applied()
}

fn last_years_papers() -> Vec<Paper<'static>> {
    vec![
        paper("chimney", "Chimney Sweep Ltd", date(2024, 11, 3), Some((2024, TaxArea::HouseholdServices))),
        paper("charity", "Sea Rescue", date(2024, 12, 20), Some((2024, TaxArea::Donations))),
        paper("untaxed", "Some Shop", date(2024, 5, 1), None),
    ]
}

#[test]
fn the_senders_of_last_year_become_items_of_this_year_besides_the_standard_list() {
    let year = build(&catalog_of(&last_years_papers()), 2025, date(2025, 6, 1));

    let mut labels: Vec<(&str, TaxArea)> = year.items.iter().map(|item| (item.label.as_str(), item.area)).collect();
    labels.sort_by_key(|(label, _)| *label);
    assert_eq!(
        labels,
        [
            ("Chimney Sweep Ltd", TaxArea::HouseholdServices),
            ("Health insurance certificate", TaxArea::PensionInsurance),
            ("Lohnsteuerbescheinigung", TaxArea::Income),
            ("Sea Rescue", TaxArea::Donations),
        ]
    );
    assert_eq!(item(&year, "Chimney Sweep Ltd").correspondent.as_deref(), Some("Chimney Sweep Ltd"));
    assert!(year.items.iter().all(|item| !item.added_by_user));
}

#[test]
fn an_item_is_expected_while_its_usual_date_is_ahead_and_missing_afterwards() {
    let catalog = catalog_of(&last_years_papers());

    let early = build(&catalog, 2025, date(2025, 11, 1));
    assert_eq!(item(&early, "Chimney Sweep Ltd").state, TaxItemState::Expected, "usually comes on 3 Nov");
    assert_eq!(item(&early, "Lohnsteuerbescheinigung").state, TaxItemState::Expected, "comes by end of February");

    let late = build(&catalog, 2025, date(2026, 3, 1));
    assert_eq!(item(&late, "Chimney Sweep Ltd").state, TaxItemState::Missing);
    assert_eq!(item(&late, "Lohnsteuerbescheinigung").state, TaxItemState::Missing);
}

#[test]
fn a_document_of_the_year_from_the_same_sender_and_area_ticks_the_item() {
    let mut papers = last_years_papers();
    papers.push(paper("chimney-25", "chimney sweep ltd ", date(2025, 11, 2), Some((2025, TaxArea::HouseholdServices))));
    papers.push(paper("other-area", "Sea Rescue", date(2025, 12, 1), Some((2025, TaxArea::Other))));

    let year = build(&catalog_of(&papers), 2025, date(2026, 3, 1));

    let chimney = item(&year, "Chimney Sweep Ltd");
    assert_eq!(chimney.state, TaxItemState::Found);
    assert_eq!(chimney.document_id, Some(test_data::document_id("chimney-25")));
    assert_eq!(item(&year, "Sea Rescue").state, TaxItemState::Missing, "only the same area counts");
    assert!(year.items.iter().any(|item| item.area == TaxArea::Other && item.state == TaxItemState::Found));
}

#[test]
fn a_standard_item_is_found_by_its_label_in_the_title() {
    let certificate = Paper {
        id: "wage",
        sender: None,
        title: Some("LOHNSTEUERBESCHEINIGUNG 2025"),
        on: date(2026, 2, 10),
        tax: Some((2025, TaxArea::Income)),
    };

    let year = build(&catalog_of(&[certificate]), 2025, date(2026, 3, 1));

    assert_eq!(item(&year, "Lohnsteuerbescheinigung").state, TaxItemState::Found);
}

#[test]
fn senders_expected_by_their_rhythm_in_the_filing_period_are_listed_in_their_tax_area() {
    let papers = [
        paper("p1", "Pension Fund", date(2023, 2, 14), Some((2022, TaxArea::PensionInsurance))),
        paper("p2", "Pension Fund", date(2024, 2, 14), None),
        paper("p3", "Pension Fund", date(2025, 2, 14), None),
    ];

    let year = build(&catalog_of(&papers), 2025, date(2025, 12, 1));

    let pension = item(&year, "Pension Fund");
    assert_eq!(pension.area, TaxArea::PensionInsurance);
    assert_eq!(pension.state, TaxItemState::Expected, "expected again in February 2026");
}

#[test]
fn rebuilding_keeps_ids_labels_user_items_and_not_needed_states() {
    let catalog = catalog_of(&last_years_papers());
    let mut first = build(&catalog, 2025, date(2025, 6, 1));
    let chimney_id = item(&first, "Chimney Sweep Ltd").id.clone();
    for item in &mut first.items {
        if item.label == "Chimney Sweep Ltd" {
            item.label = "Chimney sweep".to_owned();
        }
        if item.label == "Sea Rescue" {
            item.state = TaxItemState::NotNeeded;
        }
    }
    first.items.push(TaxChecklistItem {
        id: TaxItemId::new("mine").expect("valid id"),
        area: TaxArea::WorkExpenses,
        label: "Laptop".to_owned(),
        correspondent: None,
        document_id: None,
        state: TaxItemState::Missing,
        note: Some("bought in May".to_owned()),
        added_by_user: true,
    });
    let stored = with_year(&catalog, first.clone());

    let again =
        TaxChecklistBuilder::build(&stored, 2025, date(2025, 6, 1), &mut || TaxItemId::new("fresh").expect("valid id"));

    assert_eq!(again.items.len(), first.items.len(), "nothing added twice, nothing removed");
    assert_eq!(item(&again, "Chimney sweep").id, chimney_id);
    assert_eq!(item(&again, "Sea Rescue").state, TaxItemState::NotNeeded);
    assert_eq!(item(&again, "Laptop").note.as_deref(), Some("bought in May"));
    assert_eq!(again, first, "an unchanged library builds the same checklist");
}

#[test]
fn a_linked_document_stays_while_it_is_assigned_to_the_year_and_area() {
    let mut papers = last_years_papers();
    papers.push(paper("receipt", "Other Name", date(2025, 3, 1), Some((2025, TaxArea::Donations))));
    let catalog = catalog_of(&papers);
    let mut year = build(&catalog, 2025, date(2025, 6, 1));
    for item in &mut year.items {
        if item.label == "Sea Rescue" {
            item.document_id = Some(test_data::document_id("receipt"));
        }
    }
    let stored = with_year(&catalog, year);

    assert_eq!(item(&build(&stored, 2025, date(2025, 6, 1)), "Sea Rescue").state, TaxItemState::Found);

    let moved = stored
        .set_tax(
            &test_data::document_id("receipt"),
            Some(TaxAssignment { year: 2024, area: TaxArea::Donations, deductible: None }),
        )
        .applied();
    let rebuilt = build(&moved, 2025, date(2026, 3, 1));
    assert_eq!(item(&rebuilt, "Sea Rescue").document_id, None);
    assert_eq!(item(&rebuilt, "Sea Rescue").state, TaxItemState::Missing);
}

#[test]
fn sums_per_area_use_the_deductible_part_or_else_the_amount_field() {
    let mut labour = document_of(&paper("plumber", "Plumber", date(2025, 4, 1), None));
    labour.tax =
        Some(TaxAssignment { year: 2025, area: TaxArea::HouseholdServices, deductible: Some(euros("684.50")) });
    labour.fields.insert(FieldPresets::amount().key, money_value("1200.00"));
    let mut sweep = document_of(&paper("sweep", "Sweep", date(2025, 5, 1), Some((2025, TaxArea::HouseholdServices))));
    sweep.fields.insert(FieldPresets::amount().key, money_value("96.00"));
    let mut old = document_of(&paper("old", "Sweep", date(2024, 5, 1), Some((2024, TaxArea::HouseholdServices))));
    old.fields.insert(FieldPresets::amount().key, money_value("90.00"));
    let catalog = test_data::catalog_with(&[labour, sweep, old]);

    let sums = TaxSums::of(&catalog, 2025);

    assert_eq!(sums.keys().copied().collect::<Vec<_>>(), [TaxArea::HouseholdServices]);
    assert_eq!(sums[&TaxArea::HouseholdServices].amounts(), [euros("780.50")]);
}

fn money_value(amount: &str) -> FieldValue {
    let Money { amount, currency } = euros(amount);
    FieldValue::Money { amount, currency }
}

#[test]
fn the_tax_filter_matches_assignments_and_falls_back_to_the_tag_for_unassigned_documents() {
    let mut tagged = document_of(&paper("tagged", "Shop", date(2025, 7, 1), None));
    tagged.tags.insert(tag("Tax"));
    let mut tagged_old = document_of(&paper("tagged-old", "Shop", date(2024, 7, 1), None));
    tagged_old.tags.insert(tag("Tax"));
    let mut assigned_elsewhere = document_of(&paper("moved", "Shop", date(2025, 7, 1), Some((2024, TaxArea::Other))));
    assigned_elsewhere.tags.insert(tag("Tax"));
    let assigned = document_of(&paper("assigned", "Shop", date(2026, 1, 10), Some((2025, TaxArea::Donations))));
    let catalog = test_data::catalog_with(&[tagged, tagged_old, assigned_elsewhere, assigned]);
    let context = QueryContext { catalog: &catalog, today: date(2026, 1, 20), zone: chrono_tz::Europe::Berlin };
    let ids_of = |filter: TaxFilter| {
        let query = DocumentQuery { tax: Some(filter), ..DocumentQuery::default() };
        query.filter(&context).iter().map(|document| document.id.value().to_owned()).collect::<Vec<_>>()
    };

    let preset = TaxFilter { year: Some(2025), area: None, tag_fallback: Some(tag("Tax")) };
    assert_eq!(ids_of(preset), ["assigned", "tagged"]);
    let by_area = TaxFilter { year: None, area: Some(TaxArea::Donations), tag_fallback: Some(tag("Tax")) };
    assert_eq!(ids_of(by_area), ["assigned"]);
}
