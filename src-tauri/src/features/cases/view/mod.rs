//! The presenters of the cases screen, the quick look's "Cases" row and the case part of a review: pure functions
//! from the catalog to DTOs.

mod choices;
mod dashboard_rows;
mod detail;
mod list;
#[cfg(test)]
mod tests;

pub use choices::{cases_of_document, document_candidates, review_cases};
pub use dashboard_rows::{follow_up_buttons, waiting_buttons};
pub use list::present_cases;

use dms_domain::cases::{Case, CaseStatus};
use dms_domain::catalog::Catalog;
use dms_domain::document::DocumentId;

use crate::features::cases::dto::{CaseChoiceDto, CaseStatusDto};
use crate::features::document::view::details::display_name;

pub(crate) fn status_dto(status: CaseStatus) -> CaseStatusDto {
    match status {
        CaseStatus::Open => CaseStatusDto::Open,
        CaseStatus::Waiting => CaseStatusDto::Waiting,
        CaseStatus::Done => CaseStatusDto::Done,
    }
}

pub(crate) fn status_of(status: CaseStatusDto) -> CaseStatus {
    match status {
        CaseStatusDto::Open => CaseStatus::Open,
        CaseStatusDto::Waiting => CaseStatus::Waiting,
        CaseStatusDto::Done => CaseStatus::Done,
    }
}

fn choice_of(case: &Case) -> CaseChoiceDto {
    CaseChoiceDto { id: case.id.to_string(), title: case.title.clone(), status: status_dto(case.status) }
}

/// The title of a document, or its file name; the id when it left the catalog.
fn document_name(catalog: &Catalog, id: &DocumentId) -> String {
    catalog.documents().get(id).map_or_else(|| id.to_string(), |document| display_name(document).to_owned())
}
