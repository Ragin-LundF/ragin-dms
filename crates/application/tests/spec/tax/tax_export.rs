use dms_application::catalog::CatalogEdit;
use dms_application::export::{ExportOptions, ExportScope, MISSING_ITEMS_PAGE};
use dms_application::port::system::Clock;
use dms_application::search::{FindOutcome, SavedSearchPresets};
use dms_domain::query::DocumentQuery;
use dms_domain::tax::{TaxArea, TaxFilter};
use indexmap::IndexSet;

use super::tax_kit::{YEAR, add_tax_document, assignment, tax_library};
use crate::support::test_data::{date, done, tag};

#[test]
fn a_tax_export_groups_by_area_and_starts_with_the_missing_items_page_and_sheet() {
    let library = tax_library();
    add_tax_document(
        &library,
        "old/sweep.pdf",
        "Chimney Sweep",
        date(2022, 11, 3),
        Some(assignment(2022, TaxArea::HouseholdServices)),
        None,
    );
    add_tax_document(
        &library,
        "gifts/receipt.pdf",
        "Sea Rescue",
        date(2023, 12, 1),
        Some(assignment(YEAR, TaxArea::Donations)),
        None,
    );
    let query =
        DocumentQuery { tax: Some(TaxFilter { year: Some(YEAR), ..TaxFilter::default() }), ..DocumentQuery::default() };

    library.export_documents().run(
        &ExportScope::Query(query),
        "/outside",
        "Tax 2023",
        ExportOptions { zip: true, tax_year: Some(YEAR), ..ExportOptions::default() },
    );

    let written = library.exports.written.get();
    let (bundle, _) = &written[0];
    let targets: Vec<&str> = bundle.entries.iter().map(|entry| entry.target.as_str()).collect();
    assert_eq!(targets, ["Donations/receipt.pdf"]);
    assert_eq!(bundle.files[0].target, MISSING_ITEMS_PAGE);
    let page = String::from_utf8(bundle.files[0].content.clone()).expect("text of the fake page");
    assert!(page.contains("Tax year 2023: missing items") && page.contains("Chimney Sweep"), "{page}");
    let missing: Vec<(&str, &str)> = bundle
        .missing_items
        .as_ref()
        .expect("sheet")
        .iter()
        .map(|item| (item.area.as_str(), item.label.as_str()))
        .collect();
    assert_eq!(
        missing,
        [
            ("Income", "Lohnsteuerbescheinigung"),
            ("Insurance and pension", "Health insurance certificate"),
            ("Household services and craftsmen", "Chimney Sweep"),
        ]
    );
    assert!(!library.catalog().tax_years().contains_key(&YEAR), "the export only reads");
}

#[test]
fn the_tax_preset_prefers_the_tax_year_and_falls_back_to_the_tax_tag() {
    let library = tax_library();
    let assigned = add_tax_document(
        &library,
        "assigned.pdf",
        "A",
        date(2025, 1, 10),
        Some(assignment(2024, TaxArea::Donations)),
        None,
    );
    let tagged = add_tax_document(&library, "tagged.pdf", "B", date(2024, 6, 1), None, None);
    let elsewhere = add_tax_document(
        &library,
        "elsewhere.pdf",
        "C",
        date(2024, 6, 1),
        Some(assignment(2023, TaxArea::Other)),
        None,
    );
    for document in [&tagged, &elsewhere] {
        done(
            library
                .edit_catalog()
                .run(&CatalogEdit::AddTags { document_id: document.id.clone(), tags: IndexSet::from([tag("Tax")]) }),
        );
    }

    let presets = SavedSearchPresets::of(library.clock.today(), library.ids.as_ref(), &tag("Tax"));
    let tax = presets.iter().find(|preset| preset.name == "Tax 2024").expect("tax preset");
    let FindOutcome::Found(found) = library.find_documents().run(&tax.query) else { panic!("expected Found") };

    let ids: Vec<_> = found.into_iter().map(|document| document.id).collect();
    assert_eq!(ids, [assigned.id, tagged.id]);
}
