use chrono::{Days, NaiveDate};
use dms_domain::cases::{Case, CaseStatus};
use dms_domain::catalog::Catalog;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{Document, DocumentId};
use dms_domain::{iso_date, text};

use super::choice_of;
use crate::features::cases::dto::{CasesFollowUpDto, CasesOfDocumentDto, CasesReviewDto};
use crate::features::document::dto::LinkedDocument;
use crate::features::document::view::details::{candidate_of, display_name, searchable_text};

/// Days after today a requested action without a date of its own is followed up.
const FOLLOW_UP_WITHOUT_DATE: u64 = 14;
/// How many documents "Add document" offers.
const CANDIDATES: usize = 30;

/// The cases `document` is in, the active cases it could join (those of its correspondent first) and whether it
/// has links to start a case from.
pub fn cases_of_document(catalog: &Catalog, document: &Document) -> CasesOfDocumentDto {
    let (member, other): (Vec<&Case>, Vec<&Case>) =
        catalog.cases().values().partition(|case| case.referenced_documents().any(|id| id == &document.id));
    let correspondent = document.metadata.correspondent.as_deref();
    let mut candidates: Vec<&Case> = other.into_iter().filter(|case| case.status != CaseStatus::Done).collect();
    candidates.sort_by_key(|case| !same_correspondent(case, correspondent));
    CasesOfDocumentDto {
        cases: member.into_iter().map(choice_of).collect(),
        candidates: candidates.into_iter().map(choice_of).collect(),
        has_links: !catalog.links_of(&document.id).is_empty(),
    }
}

/// The case part of the review of `document`: the actions the letter requests (followed up on their date, or in
/// two weeks), the documents that match them, and the active case of the same correspondent to add it to.
pub fn review_cases(
    catalog: &Catalog,
    document: &Document,
    suggestion: Option<&ClassificationSuggestion>,
    matches: &[DocumentId],
    today: NaiveDate,
) -> CasesReviewDto {
    let correspondent = suggestion
        .and_then(|suggestion| suggestion.correspondent.as_deref())
        .or(document.metadata.correspondent.as_deref());
    let open_case = open_case_of(catalog, &document.id, correspondent);
    let later = today.checked_add_days(Days::new(FOLLOW_UP_WITHOUT_DATE)).unwrap_or(today);
    CasesReviewDto {
        requested: suggestion
            .map(|suggestion| suggestion.requested_actions.as_slice())
            .unwrap_or_default()
            .iter()
            .map(|action| CasesFollowUpDto {
                text: action.text.clone(),
                date: iso_date(action.due_date.unwrap_or(later)),
            })
            .collect(),
        matches: matches
            .iter()
            .filter(|id| *id != &document.id)
            .filter_map(|id| catalog.documents().get(id))
            .map(candidate_of)
            .collect(),
        pre_checked: open_case.is_some_and(|case| case.status == CaseStatus::Waiting),
        open_case: open_case.map(choice_of),
        new_title: suggestion
            .and_then(|suggestion| suggestion.title.clone())
            .unwrap_or_else(|| display_name(document).to_owned()),
    }
}

/// Documents not yet in `case` whose path, title or correspondent contains `query`, newest first.
pub fn document_candidates(catalog: &Catalog, case: &Case, query: &str) -> Vec<LinkedDocument> {
    let wanted = text::lowercase(text::trim(query));
    let mut matches: Vec<&Document> = catalog
        .documents()
        .values()
        .filter(|document| !case.referenced_documents().any(|id| id == &document.id))
        .filter(|document| wanted.is_empty() || searchable_text(document).contains(&wanted))
        .collect();
    matches.sort_by_key(|document| std::cmp::Reverse(document.added_at));
    matches.into_iter().take(CANDIDATES).map(candidate_of).collect()
}

/// The correspondent's active case without the document: one waiting for an answer first, then the newest.
fn open_case_of<'c>(catalog: &'c Catalog, document: &DocumentId, correspondent: Option<&str>) -> Option<&'c Case> {
    correspondent?;
    catalog
        .cases()
        .values()
        .filter(|case| case.status != CaseStatus::Done && same_correspondent(case, correspondent))
        .filter(|case| !case.referenced_documents().any(|id| id == document))
        .max_by_key(|case| (case.status == CaseStatus::Waiting, case.opened_on))
}

fn same_correspondent(case: &Case, correspondent: Option<&str>) -> bool {
    let key = |name: &str| text::lowercase(text::trim(name));
    matches!((case.correspondent.as_deref(), correspondent), (Some(own), Some(other)) if key(own) == key(other))
}
