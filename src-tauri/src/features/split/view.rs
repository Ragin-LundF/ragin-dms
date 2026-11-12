//! The proposal as the dialog shows it, and the dialog's parts back as domain values.

use dms_application::split::ProposeSplitOutcome;
use dms_domain::catalog::Catalog;
use dms_domain::classification::SuggestedCategory;
use dms_domain::document::Document;
use dms_domain::split::SplitPart;
use dms_domain::taxonomy::CategoryId;
use dms_domain::{iso_date, parse_iso_date};

use super::dto::{SplitPartDto, SplitProposalDto};
use crate::dto::IpcError;
use crate::features::library::view::tokens::category_path;
use crate::messages;

/// The proposal, or the reason there is none as an error for the dialog.
pub fn proposal_dto(
    outcome: ProposeSplitOutcome,
    document: &Document,
    catalog: &Catalog,
) -> Result<SplitProposalDto, IpcError> {
    let ProposeSplitOutcome::Proposed { proposal, model, page_count, .. } = outcome else {
        let problem = messages::split_problem(&outcome).unwrap_or_default();
        return Err(IpcError::invalid(problem));
    };
    Ok(SplitProposalDto {
        document_id: document.id.value().to_owned(),
        name: document.path.file_name().to_owned(),
        page_count: count(page_count),
        model,
        parts: proposal.parts.iter().map(|part| part_dto(part, catalog)).collect(),
        note: proposal.problem.map(|problem| messages::split_fallback(problem).to_owned()),
    })
}

fn part_dto(part: &SplitPart, catalog: &Catalog) -> SplitPartDto {
    let (category_id, new_category, category_label) = match &part.category {
        None => (None, None, None),
        Some(SuggestedCategory::Existing(id)) => (Some(id.value().to_owned()), None, Some(category_path(id, catalog))),
        Some(SuggestedCategory::New { name, .. }) => (None, Some(name.clone()), Some(format!("{name} (new)"))),
    };
    SplitPartDto {
        first_page: count(part.first_page),
        last_page: count(part.last_page),
        title: part.title.clone(),
        category_id,
        new_category,
        category_label,
        correspondent: part.correspondent.clone(),
        document_date: part.document_date.map(iso_date),
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0..=1 checked by the adapter
        confidence: part.confidence.map(|confidence| (confidence * 100.0).round() as u32),
    }
}

/// The dialog's part; an unknown category id or a date that is none is dropped, not refused.
pub fn part_of(dto: &SplitPartDto) -> SplitPart {
    let category = match (&dto.category_id, &dto.new_category) {
        (Some(id), _) => CategoryId::new(id.as_str()).ok().map(SuggestedCategory::Existing),
        (None, Some(name)) => Some(SuggestedCategory::New { name: name.clone(), parent_id: None }),
        (None, None) => None,
    };
    SplitPart {
        first_page: dto.first_page as usize,
        last_page: dto.last_page as usize,
        title: dto.title.clone(),
        category,
        correspondent: dto.correspondent.clone(),
        document_date: dto.document_date.as_deref().and_then(parse_iso_date),
        confidence: dto.confidence.map(|percent| f64::from(percent.min(100)) / 100.0),
    }
}

fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use dms_application::split::PageSource;
    use dms_domain::catalog::{CatalogUpdate, LibraryId};
    use dms_domain::document::{ContentHash, DocumentId};
    use dms_domain::library::RelativePath;
    use dms_domain::split::SplitProposal;
    use dms_domain::taxonomy::Category;

    fn applied(update: CatalogUpdate) -> Catalog {
        match update {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
        }
    }

    fn id(raw: &str) -> CategoryId {
        CategoryId::new(raw).expect("valid id")
    }

    #[test]
    fn a_proposal_shows_category_paths_new_categories_and_percentages_and_comes_back_unchanged() {
        let home = Category::new(id("c-home"), "Home", None, None).expect("category");
        let energy = Category::new(id("c-energy"), "Energy", Some(id("c-home")), None).expect("category");
        let document = Document::new(
            DocumentId::new("d-1").expect("id"),
            RelativePath::of("Inbox/stack.pdf").expect("path"),
            ContentHash::new(format!("{:064x}", 1)).expect("hash"),
            100,
            "2026-01-01T10:00:00Z".parse().expect("instant"),
        )
        .expect("document");
        let mut catalog = Catalog::empty(LibraryId::new("library-1").expect("id"));
        for category in [home, energy] {
            catalog = applied(catalog.add_category(category).expect("valid"));
        }
        let catalog = applied(catalog.add_document(document.clone()).expect("valid"));
        let parts = vec![
            SplitPart {
                title: Some("Electricity bill".to_owned()),
                category: Some(SuggestedCategory::Existing(id("c-energy"))),
                document_date: NaiveDate::from_ymd_opt(2026, 3, 2),
                confidence: Some(0.93),
                ..SplitPart::pages(0, 1)
            },
            SplitPart {
                category: Some(SuggestedCategory::New { name: "Tax".to_owned(), parent_id: None }),
                ..SplitPart::pages(2, 2)
            },
        ];
        let outcome = ProposeSplitOutcome::Proposed {
            proposal: SplitProposal { parts: parts.clone(), problem: None },
            model: "m".to_owned(),
            page_count: 3,
            source: PageSource::TextLayer,
        };

        let dto = proposal_dto(outcome, &document, &catalog).expect("a proposal");

        assert_eq!((dto.name.as_str(), dto.page_count, dto.note.clone()), ("stack.pdf", 3, None));
        assert_eq!(dto.parts[0].category_label.as_deref(), Some("Home › Energy"));
        assert_eq!(dto.parts[0].confidence, Some(93));
        assert_eq!(dto.parts[0].document_date.as_deref(), Some("2026-03-02"));
        assert_eq!(dto.parts[1].category_label.as_deref(), Some("Tax (new)"));
        assert_eq!(dto.parts.iter().map(part_of).collect::<Vec<_>>(), parts);
    }

    #[test]
    fn no_proposal_is_an_error_with_the_reason() {
        let document = Document::new(
            DocumentId::new("d-1").expect("id"),
            RelativePath::of("photo.jpg").expect("path"),
            ContentHash::new(format!("{:064x}", 1)).expect("hash"),
            100,
            "2026-01-01T10:00:00Z".parse().expect("instant"),
        )
        .expect("document");
        let catalog = Catalog::empty(LibraryId::new("library-1").expect("id"));

        let error = proposal_dto(ProposeSplitOutcome::NotPdf, &document, &catalog).expect_err("no proposal");

        assert_eq!(error.message, "Only PDFs can be split");
    }
}
