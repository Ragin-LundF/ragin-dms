use dms_domain::DomainError;
use dms_domain::cases::{Case, CaseId, FollowUp, FollowUpId};
use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::document::DocumentId;

use crate::cases::NewFollowUp;
use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::port::system::IdGenerator;

/// Saves the case `case_id` as `change` returns it. A case that is gone is rejected; a `change` that answers `None`
/// (the follow-up it looks for is gone) is refused as invalid input.
pub(crate) fn edit_case(
    store: &CatalogStore,
    case_id: &CaseId,
    change: impl Fn(&Catalog, &Case) -> Option<Case>,
) -> OperationOutcome {
    store.commit(|catalog| {
        let Some(case) = catalog.cases().get(case_id) else {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::CaseNotFound));
        };
        let changed = change(catalog, case)
            .ok_or_else(|| DomainError::InvalidArgument("The follow-up no longer exists".to_owned()))?;
        catalog.save_case(changed)
    })
}

/// The day a document joins a case: its document date, or `today` when it has none or is not catalogued.
pub(crate) fn entry_date(catalog: &Catalog, document: &DocumentId, today: chrono::NaiveDate) -> chrono::NaiveDate {
    catalog.documents().get(document).and_then(|document| document.metadata.document_date).unwrap_or(today)
}

pub(crate) fn follow_up_of(new: &NewFollowUp, ids: &dyn IdGenerator) -> FollowUp {
    FollowUp {
        id: FollowUpId::new(ids.new_id()).expect("invariant: generated ids are not blank"),
        date: new.date,
        text: dms_domain::text::trim(&new.text).to_owned(),
        done: false,
        source: new.source.clone(),
    }
}
