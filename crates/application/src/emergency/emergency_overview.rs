use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::emergency::{EmergencyEntry, EmergencyFolder, EmergencyGroup, EmergencyItem};
use dms_domain::settings::OwnerDetails;

use crate::port::pdf::{TextBlock, TextDocument};

const GROUP_ORDER: [EmergencyGroup; 6] = [
    EmergencyGroup::Contacts,
    EmergencyGroup::Insurance,
    EmergencyGroup::Accounts,
    EmergencyGroup::Contracts,
    EmergencyGroup::Personal,
    EmergencyGroup::Other,
];

/// The printable overview of the emergency folder: title, owner and date, the intro and one table row per included
/// item, grouped in the fixed group order.
pub fn emergency_overview(
    folder: &EmergencyFolder,
    catalog: &Catalog,
    owner: Option<&OwnerDetails>,
    today: NaiveDate,
) -> TextDocument {
    let mut blocks = vec![TextBlock::Heading("Emergency folder".to_owned())];
    let as_of = format!("as of {}", today.format("%Y-%m-%d"));
    blocks.push(TextBlock::Paragraph(match owner {
        Some(owner) => format!("{} · {as_of}", owner.name),
        None => as_of,
    }));
    if let Some(intro) = &folder.intro_text {
        blocks.push(TextBlock::Paragraph(intro.clone()));
    }
    blocks.push(TextBlock::Spacer);
    let rows: Vec<Vec<String>> = included_in_order(folder)
        .map(|item| vec![group_label(item.group).to_owned(), item_text(&item.entry, catalog)])
        .collect();
    blocks.push(TextBlock::Table { header: vec!["Area".to_owned(), "Document / contact".to_owned()], rows });
    TextDocument { blocks }
}

/// The included items, group by group in the fixed order, each group in the folder's order.
pub(crate) fn included_in_order(folder: &EmergencyFolder) -> impl Iterator<Item = &EmergencyItem> {
    GROUP_ORDER.iter().flat_map(|group| folder.items.iter().filter(move |item| item.included && item.group == *group))
}

pub fn group_label(group: EmergencyGroup) -> &'static str {
    match group {
        EmergencyGroup::Contacts => "Contacts",
        EmergencyGroup::Insurance => "Insurance",
        EmergencyGroup::Accounts => "Accounts",
        EmergencyGroup::Contracts => "Contracts",
        EmergencyGroup::Personal => "Personal",
        EmergencyGroup::Other => "Other",
    }
}

/// One line for an item: a document's title (or file name), correspondent and date; a contract's name, provider and
/// note; a contact's name, role, phone, email and note; a note's text. A document or contract the catalog no longer
/// has is named as missing.
pub fn item_text(entry: &EmergencyEntry, catalog: &Catalog) -> String {
    match entry {
        EmergencyEntry::Document(id) => catalog.documents().get(id).map_or_else(
            || "Missing document".to_owned(),
            |document| {
                let title = document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned());
                let date = document.metadata.document_date.map(|date| date.format("%Y-%m-%d").to_string());
                joined([Some(title), document.metadata.correspondent.clone(), date])
            },
        ),
        EmergencyEntry::Contract(id) => catalog.contracts().get(id).map_or_else(
            || "Missing contract".to_owned(),
            |contract| joined([Some(contract.name.clone()), contract.correspondent.clone(), contract.note.clone()]),
        ),
        EmergencyEntry::Contact { name, role, phone, email, note } => {
            joined([Some(name.clone()), role.clone(), phone.clone(), email.clone(), note.clone()])
        }
        EmergencyEntry::Note(text) => text.clone(),
    }
}

fn joined<const N: usize>(parts: [Option<String>; N]) -> String {
    parts.into_iter().flatten().collect::<Vec<_>>().join(" · ")
}
