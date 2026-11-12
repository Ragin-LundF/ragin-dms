//! The presenters of the emergency screen: pure functions from domain types to DTOs.

use std::path::Path;

use chrono::NaiveDate;
use dms_application::emergency::{EmergencyEdit, emergency_overview, group_label, item_text};
use dms_application::port::pdf::TextBlock;
use dms_domain::catalog::Catalog;
use dms_domain::contract::ContractId;
use dms_domain::document::DocumentId;
use dms_domain::emergency::{
    EmergencyChanges, EmergencyEntry, EmergencyFolder, EmergencyGroup, EmergencyItem, EmergencyItemId, EmergencyPlanner,
};
use dms_domain::text;

use super::dto::{
    EmergencyChoiceDto, EmergencyEditDto, EmergencyGroupDto, EmergencyGroupListDto, EmergencyItemDto,
    EmergencyItemKindDto, EmergencyMembershipDto, EmergencyPreviewDto, EmergencyPreviewRowDto, EmergencyStateDto,
    EmergencyStatusDto,
};
use crate::features::letters::view::folders_of;

const GROUPS: [EmergencyGroup; 6] = [
    EmergencyGroup::Contacts,
    EmergencyGroup::Insurance,
    EmergencyGroup::Accounts,
    EmergencyGroup::Contracts,
    EmergencyGroup::Personal,
    EmergencyGroup::Other,
];
const CANDIDATES: usize = 20;

pub fn present_emergency(catalog: &Catalog, today: NaiveDate) -> EmergencyStateDto {
    let folder = catalog.settings().emergency_folder.clone().unwrap_or_default();
    let changed = EmergencyChanges::of(&folder, catalog);
    let groups = GROUPS
        .iter()
        .map(|group| EmergencyGroupListDto {
            group: group_dto(*group),
            label: group_label(*group).to_owned(),
            items: folder
                .items
                .iter()
                .filter(|item| item.group == *group)
                .map(|item| item_dto(item, catalog, changed.contains(&item.id)))
                .collect(),
        })
        .collect();
    let in_folder: Vec<&ContractId> = folder.referenced_contracts().collect();
    EmergencyStateDto {
        status: status(&folder, catalog, today, changed.len()),
        refresh_months: folder.refresh_months,
        intro: folder.intro_text.clone().unwrap_or_default(),
        groups,
        preview: preview(&folder, catalog, today),
        included: u32::try_from(folder.items.iter().filter(|item| item.included).count()).unwrap_or(u32::MAX),
        contracts: catalog
            .contracts()
            .values()
            .filter(|contract| !in_folder.contains(&&contract.id))
            .map(|contract| EmergencyChoiceDto {
                id: contract.id.value().to_owned(),
                name: contract.name.clone(),
                detail: contract.correspondent.clone().unwrap_or_default(),
            })
            .collect(),
        folders: folders_of(catalog),
    }
}

fn status(folder: &EmergencyFolder, catalog: &Catalog, today: NaiveDate, changed: usize) -> EmergencyStatusDto {
    let changed_count = u32::try_from(changed).unwrap_or(u32::MAX);
    let (Some(refreshed), Some(next)) = (folder.last_refreshed, EmergencyPlanner::next_refresh(folder)) else {
        let detail = if folder.items.is_empty() {
            "Add documents, contracts, contacts and notes"
        } else {
            "Go through the items, then mark the folder as refreshed"
        };
        return EmergencyStatusDto {
            title: "Not refreshed yet".to_owned(),
            detail: detail.to_owned(),
            refresh_due: false,
            changed: changed_count,
        };
    };
    let detail = match changed {
        0 => format!("Next refresh on {}", iso(next)),
        changed => format!("Next refresh on {} · {changed} changed since then", iso(next)),
    };
    EmergencyStatusDto {
        title: format!("Last refreshed {}", iso(refreshed)),
        detail,
        refresh_due: !EmergencyPlanner::due_tasks(catalog, today).is_empty(),
        changed: changed_count,
    }
}

fn item_dto(item: &EmergencyItem, catalog: &Catalog, changed: bool) -> EmergencyItemDto {
    let (kind, document_id) = match &item.entry {
        EmergencyEntry::Document(id) => (EmergencyItemKindDto::Document, Some(id.value().to_owned())),
        EmergencyEntry::Contract(_) => (EmergencyItemKindDto::Contract, None),
        EmergencyEntry::Contact { .. } => (EmergencyItemKindDto::Contact, None),
        EmergencyEntry::Note(_) => (EmergencyItemKindDto::Note, None),
    };
    EmergencyItemDto {
        id: item.id.value().to_owned(),
        kind,
        text: item_text(&item.entry, catalog),
        included: item.included,
        changed,
        document_id,
    }
}

/// The overview's first blocks as the screen previews them; the same document the PDF is written from.
fn preview(folder: &EmergencyFolder, catalog: &Catalog, today: NaiveDate) -> EmergencyPreviewDto {
    let overview = emergency_overview(folder, catalog, catalog.settings().owner.as_ref(), today);
    let mut preview =
        EmergencyPreviewDto { title: String::new(), subtitle: String::new(), intro: String::new(), rows: Vec::new() };
    let mut paragraphs = 0;
    for block in overview.blocks {
        match block {
            TextBlock::Heading(title) => preview.title = title,
            TextBlock::Paragraph(text) => {
                if paragraphs == 0 {
                    preview.subtitle = text;
                } else {
                    preview.intro = text;
                }
                paragraphs += 1;
            }
            TextBlock::Table { rows, .. } => {
                preview.rows = rows
                    .into_iter()
                    .map(|row| {
                        let mut cells = row.into_iter();
                        let area = cells.next().unwrap_or_default();
                        EmergencyPreviewRowDto { area, text: cells.next().unwrap_or_default() }
                    })
                    .collect();
            }
            TextBlock::KeyValueTable(_) | TextBlock::Spacer => {}
        }
    }
    preview
}

/// The use case's edit for what the window sent; the message when an id is no id.
pub fn edit_of(edit: &EmergencyEditDto) -> Result<EmergencyEdit, String> {
    let item_id = |raw: &str| EmergencyItemId::new(raw).map_err(|error| error.to_string());
    Ok(match edit {
        EmergencyEditDto::AddDocument { group, document_id } => EmergencyEdit::Add {
            group: group_of(*group),
            entry: EmergencyEntry::Document(DocumentId::new(document_id).map_err(|error| error.to_string())?),
        },
        EmergencyEditDto::AddContract { group, contract_id } => EmergencyEdit::Add {
            group: group_of(*group),
            entry: EmergencyEntry::Contract(ContractId::new(contract_id).map_err(|error| error.to_string())?),
        },
        EmergencyEditDto::AddContact { group, name, role, phone, email, note } => EmergencyEdit::Add {
            group: group_of(*group),
            entry: EmergencyEntry::Contact {
                name: text::trim(name).to_owned(),
                role: optional(role),
                phone: optional(phone),
                email: optional(email),
                note: optional(note),
            },
        },
        EmergencyEditDto::AddNote { group, text } => {
            EmergencyEdit::Add { group: group_of(*group), entry: EmergencyEntry::Note(text::trim(text).to_owned()) }
        }
        EmergencyEditDto::Remove { item_id: id } => EmergencyEdit::Remove(item_id(id)?),
        EmergencyEditDto::SetIncluded { item_id: id, included } => {
            EmergencyEdit::SetIncluded { id: item_id(id)?, included: *included }
        }
        EmergencyEditDto::SetIntro { intro } => EmergencyEdit::SetIntro(optional(intro)),
        EmergencyEditDto::SetRefreshMonths { months } => EmergencyEdit::SetRefreshMonths(*months),
    })
}

/// The item of the document or contract, if it is in the emergency folder.
pub fn membership(catalog: &Catalog, entry: &EmergencyEntry) -> EmergencyMembershipDto {
    let item = catalog
        .settings()
        .emergency_folder
        .as_ref()
        .and_then(|folder| folder.items.iter().find(|item| &item.entry == entry));
    EmergencyMembershipDto {
        item_id: item.map(|item| item.id.value().to_owned()),
        group: item.map(|item| group_label(item.group).to_owned()),
    }
}

/// Documents matching `query` in name, title or correspondent that are not in the folder yet, newest first.
pub fn document_candidates(catalog: &Catalog, query: &str) -> Vec<EmergencyChoiceDto> {
    let wanted = text::lowercase(text::trim(query));
    let in_folder: Vec<&DocumentId> =
        catalog.settings().emergency_folder.iter().flat_map(EmergencyFolder::referenced_documents).collect();
    let mut matches: Vec<_> = catalog
        .documents()
        .values()
        .filter(|document| !in_folder.contains(&&document.id))
        .filter(|document| {
            let parts = [
                Some(document.path.value()),
                document.metadata.title.as_deref(),
                document.metadata.correspondent.as_deref(),
            ];
            wanted.is_empty() || parts.into_iter().flatten().any(|part| text::lowercase(part).contains(&wanted))
        })
        .collect();
    matches.sort_by_key(|document| std::cmp::Reverse(document.added_at));
    matches
        .into_iter()
        .take(CANDIDATES)
        .map(|document| EmergencyChoiceDto {
            id: document.id.value().to_owned(),
            name: document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned()),
            detail: document.path.value().to_owned(),
        })
        .collect()
}

/// The folder and the file name of a path from the save dialog; `None` for a path without both.
pub fn split_path(path: &str) -> Option<(String, String)> {
    let path = Path::new(path);
    let directory = path.parent()?.to_string_lossy().into_owned();
    let name = path.file_name()?.to_string_lossy().into_owned();
    (!directory.is_empty()).then_some((directory, name))
}

fn optional(value: &str) -> Option<String> {
    Some(text::trim(value).to_owned()).filter(|value| !value.is_empty())
}

fn iso(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn group_dto(group: EmergencyGroup) -> EmergencyGroupDto {
    match group {
        EmergencyGroup::Contacts => EmergencyGroupDto::Contacts,
        EmergencyGroup::Insurance => EmergencyGroupDto::Insurance,
        EmergencyGroup::Accounts => EmergencyGroupDto::Accounts,
        EmergencyGroup::Contracts => EmergencyGroupDto::Contracts,
        EmergencyGroup::Personal => EmergencyGroupDto::Personal,
        EmergencyGroup::Other => EmergencyGroupDto::Other,
    }
}

fn group_of(group: EmergencyGroupDto) -> EmergencyGroup {
    match group {
        EmergencyGroupDto::Contacts => EmergencyGroup::Contacts,
        EmergencyGroupDto::Insurance => EmergencyGroup::Insurance,
        EmergencyGroupDto::Accounts => EmergencyGroup::Accounts,
        EmergencyGroupDto::Contracts => EmergencyGroup::Contracts,
        EmergencyGroupDto::Personal => EmergencyGroup::Personal,
        EmergencyGroupDto::Other => EmergencyGroup::Other,
    }
}

#[cfg(test)]
mod tests {
    use dms_domain::catalog::{CatalogUpdate, LibraryId};
    use dms_domain::document::{ContentHash, Document};
    use dms_domain::library::RelativePath;
    use dms_domain::settings::CatalogSettings;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("date")
    }

    fn applied(update: Result<CatalogUpdate, dms_domain::DomainError>) -> Catalog {
        match update.expect("valid") {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
        }
    }

    fn catalog_with_folder() -> Catalog {
        let policy = Document {
            metadata: dms_domain::document::DocumentMetadata {
                title: Some("Life insurance".to_owned()),
                ..Default::default()
            },
            ..Document::new(
                DocumentId::new("d-1").expect("id"),
                RelativePath::of("insurance/policy.pdf").expect("path"),
                ContentHash::new(format!("{:064x}", 1)).expect("hash"),
                10,
                "2024-01-01T10:00:00Z".parse().expect("instant"),
            )
            .expect("document")
        };
        let catalog = applied(Catalog::empty(LibraryId::new("library-1").expect("id")).add_document(policy));
        let item = |id: &str, group, entry, recorded_hash| EmergencyItem {
            id: EmergencyItemId::new(id).expect("id"),
            group,
            entry,
            included: true,
            recorded_hash,
        };
        let folder = EmergencyFolder {
            items: vec![
                item(
                    "e-1",
                    EmergencyGroup::Insurance,
                    EmergencyEntry::Document(DocumentId::new("d-1").expect("id")),
                    Some(ContentHash::new(format!("{:064x}", 2)).expect("hash")),
                ),
                item("e-2", EmergencyGroup::Contacts, EmergencyEntry::Note("Keys at the neighbour".to_owned()), None),
            ],
            last_refreshed: Some(date(2025, 11, 14)),
            refresh_months: 12,
            intro_text: Some("Everything is in the shelf".to_owned()),
        };
        applied(
            catalog.change_settings(CatalogSettings { emergency_folder: Some(folder), ..CatalogSettings::default() }),
        )
    }

    #[test]
    fn the_screen_shows_the_status_changed_items_by_group_and_the_overview_preview() {
        let state = present_emergency(&catalog_with_folder(), date(2026, 11, 1));

        assert_eq!(state.status.title, "Last refreshed 2025-11-14");
        assert_eq!(state.status.detail, "Next refresh on 2026-11-14 · 1 changed since then");
        assert!(state.status.refresh_due);
        let contacts = &state.groups[0];
        assert_eq!((contacts.label.as_str(), contacts.items[0].text.as_str()), ("Contacts", "Keys at the neighbour"));
        let insurance = &state.groups[1].items[0];
        assert!(insurance.changed);
        assert_eq!(insurance.document_id.as_deref(), Some("d-1"));
        assert_eq!(state.preview.title, "Emergency folder");
        assert_eq!(state.preview.subtitle, "as of 2026-11-01");
        assert_eq!(state.preview.intro, "Everything is in the shelf");
        assert_eq!(
            state.preview.rows[1],
            EmergencyPreviewRowDto { area: "Insurance".to_owned(), text: "Life insurance".to_owned() }
        );
        assert_eq!(state.included, 2);
    }

    #[test]
    fn an_empty_library_has_a_folder_that_was_never_refreshed() {
        let state = present_emergency(&Catalog::empty(LibraryId::new("library-1").expect("id")), date(2026, 11, 1));

        assert_eq!(state.status.title, "Not refreshed yet");
        assert!(!state.status.refresh_due);
        assert!(state.groups.iter().all(|group| group.items.is_empty()));
    }

    #[test]
    fn candidates_leave_out_documents_in_the_folder_and_membership_names_the_group() {
        let catalog = catalog_with_folder();

        assert_eq!(document_candidates(&catalog, ""), Vec::new());
        let member = membership(&catalog, &EmergencyEntry::Document(DocumentId::new("d-1").expect("id")));
        assert_eq!((member.item_id.as_deref(), member.group.as_deref()), (Some("e-1"), Some("Insurance")));
        let other = membership(&catalog, &EmergencyEntry::Document(DocumentId::new("d-2").expect("id")));
        assert_eq!(other.item_id, None);
    }

    #[test]
    fn a_contact_is_trimmed_and_blank_details_are_left_out() {
        let edit = edit_of(&EmergencyEditDto::AddContact {
            group: EmergencyGroupDto::Contacts,
            name: " Kanzlei Berg ".to_owned(),
            role: String::new(),
            phone: "040 123456".to_owned(),
            email: " ".to_owned(),
            note: String::new(),
        });

        assert_eq!(
            edit,
            Ok(EmergencyEdit::Add {
                group: EmergencyGroup::Contacts,
                entry: EmergencyEntry::Contact {
                    name: "Kanzlei Berg".to_owned(),
                    role: None,
                    phone: Some("040 123456".to_owned()),
                    email: None,
                    note: None,
                },
            })
        );
        assert!(edit_of(&EmergencyEditDto::Remove { item_id: " ".to_owned() }).is_err());
    }

    #[test]
    fn a_save_dialog_path_splits_into_folder_and_file_name() {
        assert_eq!(split_path("/home/usb/Notfall.zip"), Some(("/home/usb".to_owned(), "Notfall.zip".to_owned())));
        assert_eq!(split_path("Notfall.zip"), None);
    }
}
