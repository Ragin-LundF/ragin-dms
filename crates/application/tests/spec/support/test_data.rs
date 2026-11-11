//! Synthetic test data. Never use real document names here.

use chrono::{DateTime, NaiveDate, Utc};
use dms_application::common::OperationOutcome;
use dms_domain::catalog::{Catalog, CatalogUpdate};
use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;
use dms_domain::taxonomy::{CategoryId, TagName};

pub fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("valid path")
}

pub fn document_id(value: &str) -> DocumentId {
    DocumentId::new(value).expect("valid id")
}

pub fn category_id(value: &str) -> CategoryId {
    CategoryId::new(value).expect("valid id")
}

pub fn tag(raw: &str) -> TagName {
    TagName::of(raw).expect("valid tag")
}

pub fn instant(text: &str) -> DateTime<Utc> {
    text.parse().expect("valid instant")
}

pub fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

/// The catalog of a [`OperationOutcome::Done`]; fails the test for every other outcome.
#[track_caller]
pub fn done(outcome: OperationOutcome) -> Catalog {
    match outcome {
        OperationOutcome::Done { catalog, .. } => catalog,
        other => panic!("expected Done, got {other:?}"),
    }
}

/// Unwraps an applied update and fails the test for a rejection or an invalid change.
pub trait Applied {
    fn applied(self) -> Catalog;
}

impl Applied for CatalogUpdate {
    #[track_caller]
    fn applied(self) -> Catalog {
        match self {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("expected an applied update, got {reason:?}"),
        }
    }
}

impl Applied for Result<CatalogUpdate, dms_domain::DomainError> {
    #[track_caller]
    fn applied(self) -> Catalog {
        self.expect("valid change").applied()
    }
}

/// `catalog` with another revision, as another device would have saved it (Kotlin's `copy(revision = …)`).
pub fn with_revision(catalog: &Catalog, revision: dms_domain::catalog::CatalogRevision) -> Catalog {
    Catalog::new(
        catalog.library_id().clone(),
        revision,
        catalog.documents().clone(),
        catalog.categories().clone(),
        catalog.tags().clone(),
        catalog.links().clone(),
        catalog.settings().clone(),
    )
    .expect("valid catalog")
}

pub fn library_id(value: &str) -> dms_domain::catalog::LibraryId {
    dms_domain::catalog::LibraryId::new(value).expect("valid id")
}
