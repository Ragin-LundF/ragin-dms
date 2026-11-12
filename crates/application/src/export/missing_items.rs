use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::iso_date;
use dms_domain::tax::{TaxArea, TaxChecklistBuilder, TaxChecklistItem, TaxItemId, TaxItemState};

use crate::port::export::{ExportFile, MissingItem};
use crate::port::pdf::{TextBlock, TextDocument};

/// The name of the missing-items page in a tax export.
pub const MISSING_ITEMS_PAGE: &str = "Missing items.pdf";

/// What a tax year's checklist still lacks on `today`: its items without a document, expected or missing, by area.
/// The checklist is built afresh, so the export never waits for the tax screen to have been opened.
pub fn missing_items_of(catalog: &Catalog, year: i32, today: NaiveDate) -> Vec<MissingItem> {
    let checklist = TaxChecklistBuilder::build(catalog, year, today, &mut || {
        TaxItemId::new("export").expect("invariant: not blank")
    });
    let mut open: Vec<&TaxChecklistItem> = checklist
        .items
        .iter()
        .filter(|item| matches!(item.state, TaxItemState::Missing | TaxItemState::Expected))
        .collect();
    open.sort_by_key(|item| TaxArea::ALL.iter().position(|area| *area == item.area));
    open.into_iter()
        .map(|item| MissingItem {
            area: item.area.label().to_owned(),
            label: item.label.clone(),
            from: item.correspondent.clone().unwrap_or_default(),
            state: state_label(item.state).to_owned(),
            note: item.note.clone().unwrap_or_default(),
        })
        .collect()
}

/// The first page of a tax export: what is still missing, so the adviser knows what is still coming.
pub fn missing_items_page(year: i32, today: NaiveDate, items: &[MissingItem]) -> TextDocument {
    let mut blocks = vec![
        TextBlock::Heading(format!("Tax year {year}: missing items")),
        TextBlock::Paragraph(format!(
            "As of {}. The documents in this export are grouped by tax area. These are still missing or expected:",
            iso_date(today)
        )),
        TextBlock::Spacer,
    ];
    if items.is_empty() {
        blocks.push(TextBlock::Paragraph("Nothing is missing.".to_owned()));
    } else {
        // Three columns, so long German words such as "Lohnsteuerbescheinigung" fit on one line.
        blocks.push(TextBlock::Table {
            header: ["Area", "Item", "State"].map(str::to_owned).to_vec(),
            rows: items.iter().map(|item| vec![item.area.clone(), item_text(item), item.state.clone()]).collect(),
        });
    }
    TextDocument { blocks }
}

/// The item's name, and its sender where the name does not say it.
fn item_text(item: &MissingItem) -> String {
    if item.from.is_empty() || item.from == item.label {
        item.label.clone()
    } else {
        format!("{} ({})", item.label, item.from)
    }
}

pub fn missing_items_file(content: Vec<u8>) -> ExportFile {
    ExportFile { target: MISSING_ITEMS_PAGE.to_owned(), content }
}

fn state_label(state: TaxItemState) -> &'static str {
    match state {
        TaxItemState::Expected => "Expected",
        TaxItemState::Missing => "Missing",
        TaxItemState::Found => "Found",
        TaxItemState::NotNeeded => "Not needed",
    }
}
