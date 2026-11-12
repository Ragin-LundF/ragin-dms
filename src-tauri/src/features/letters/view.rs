//! Between the letter use cases and the dialog's DTOs, and the folders a letter can be saved to.

use dms_application::letter::{DraftLetterOutcome, DraftSource};
use dms_application::port::assistant::{LetterKind, LetterLanguage, LetterParty};
use dms_domain::catalog::Catalog;
use dms_domain::library::RelativePath;
use dms_domain::text;
use indexmap::IndexSet;

use super::dto::{LetterDraftDto, LetterKindDto, LetterLanguageDto, LetterPartyDto};
use crate::messages;

/// The sender of letters: the owner details of the library.
pub fn letter_owner(catalog: &Catalog) -> Option<LetterParty> {
    let owner = catalog.settings().owner.as_ref()?;
    Some(LetterParty { name: owner.name.clone(), address_lines: owner.address_lines.clone() })
}

/// The dialog's draft, or `None` for an outcome without one.
pub fn draft_dto(
    outcome: DraftLetterOutcome,
    sender: Option<LetterParty>,
    catalog: &Catalog,
) -> Option<LetterDraftDto> {
    let DraftLetterOutcome::Drafted { draft, recipient, language, folder, source } = outcome else {
        return None;
    };
    Some(LetterDraftDto {
        subject: draft.subject,
        body: draft.body,
        sender: sender.map(party_dto),
        recipient: party_dto(recipient),
        language: language_dto(language),
        drafted_by_ai: matches!(source, DraftSource::Ai { .. }),
        origin: messages::letter_origin(&source),
        folder: folder.value().to_owned(),
        folders: folders_of(catalog),
    })
}

/// The top level ("") and every folder that holds documents, with the folders above them, in name order.
pub fn folders_of(catalog: &Catalog) -> Vec<String> {
    let mut folders: IndexSet<String> = IndexSet::new();
    for document in catalog.documents().values() {
        let mut parent = document.path.parent();
        while let Some(folder) = parent.filter(|folder| !folder.is_root()) {
            parent = folder.parent();
            folders.insert(folder.value().to_owned());
        }
    }
    let mut folders: Vec<String> = folders.into_iter().collect();
    folders.sort_by(|first, second| text::compare(first, second));
    folders.insert(0, RelativePath::root().value().to_owned());
    folders
}

pub fn kind_of(kind: LetterKindDto) -> LetterKind {
    match kind {
        LetterKindDto::Cancellation => LetterKind::Cancellation,
        LetterKindDto::Objection => LetterKind::Objection,
        LetterKindDto::Reminder => LetterKind::Reminder,
        LetterKindDto::Request => LetterKind::Request,
        LetterKindDto::Reply => LetterKind::Reply,
    }
}

pub fn language_of(language: LetterLanguageDto) -> LetterLanguage {
    match language {
        LetterLanguageDto::De => LetterLanguage::De,
        LetterLanguageDto::En => LetterLanguage::En,
    }
}

fn language_dto(language: LetterLanguage) -> LetterLanguageDto {
    match language {
        LetterLanguage::De => LetterLanguageDto::De,
        LetterLanguage::En => LetterLanguageDto::En,
    }
}

/// The party with trimmed lines and without empty ones.
pub fn party_of(party: &LetterPartyDto) -> LetterParty {
    let address_lines =
        party.address_lines.iter().map(|line| text::trim(line).to_owned()).filter(|line| !line.is_empty()).collect();
    LetterParty { name: text::trim(&party.name).to_owned(), address_lines }
}

fn party_dto(party: LetterParty) -> LetterPartyDto {
    LetterPartyDto { name: party.name, address_lines: party.address_lines }
}

#[cfg(test)]
mod tests {
    use dms_application::port::ai::ClassificationFailure;
    use dms_application::port::assistant::LetterDraft;
    use dms_domain::catalog::{CatalogUpdate, LibraryId};
    use dms_domain::document::{ContentHash, Document, DocumentId};

    use super::*;

    fn catalog_with(paths: &[&str]) -> Catalog {
        paths.iter().enumerate().fold(
            Catalog::empty(LibraryId::new("library-1").expect("id")),
            |catalog, (index, raw)| {
                let document = Document::new(
                    DocumentId::new(format!("d-{index}")).expect("id"),
                    RelativePath::of(raw).expect("path"),
                    ContentHash::new(format!("{index:064x}")).expect("hash"),
                    1,
                    "2024-01-01T10:00:00Z".parse().expect("instant"),
                )
                .expect("document");
                match catalog.add_document(document).expect("valid") {
                    CatalogUpdate::Applied(catalog) => catalog,
                    CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
                }
            },
        )
    }

    #[test]
    fn the_folders_are_the_top_level_and_every_folder_above_a_document_once_in_name_order() {
        let catalog = catalog_with(&["top.pdf", "Verträge/Strom/2024.pdf", "Verträge/Strom/2025.pdf", "Bank/a.pdf"]);

        assert_eq!(folders_of(&catalog), vec!["", "Bank", "Verträge", "Verträge/Strom"]);
    }

    #[test]
    fn a_template_draft_says_why_the_ai_did_not_write_it() {
        let outcome = DraftLetterOutcome::Drafted {
            draft: LetterDraft { subject: "Reminder".to_owned(), body: "Dear Sir or Madam,".to_owned() },
            recipient: LetterParty { name: "Netz".to_owned(), address_lines: Vec::new() },
            language: LetterLanguage::En,
            folder: RelativePath::of("Bank").expect("path"),
            source: DraftSource::Template { ai_failure: Some(ClassificationFailure::Timeout) },
        };

        let dto = draft_dto(outcome, None, &catalog_with(&["Bank/a.pdf"])).expect("a draft");

        assert!(!dto.drafted_by_ai);
        assert_eq!(dto.origin, "Template: the AI took too long");
        assert_eq!(dto.folder, "Bank");
        assert_eq!(dto.folders, vec!["", "Bank"]);
        assert_eq!(draft_dto(DraftLetterOutcome::NotFound, None, &catalog_with(&[])), None);
    }

    #[test]
    fn a_typed_party_loses_blank_lines_and_surrounding_spaces() {
        let party = party_of(&LetterPartyDto {
            name: " Netz GmbH ".to_owned(),
            address_lines: vec![" Postfach 1 ".to_owned(), "  ".to_owned(), "12345 Hafenstadt".to_owned()],
        });

        assert_eq!(party.name, "Netz GmbH");
        assert_eq!(party.address_lines, vec!["Postfach 1".to_owned(), "12345 Hafenstadt".to_owned()]);
    }
}
