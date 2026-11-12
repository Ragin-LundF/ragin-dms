//! Synthetic test data. Never use real document names here.

use chrono::{DateTime, NaiveDate, Utc};
use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::document::{ContentHash, Document, DocumentId};
use dms_domain::library::RelativePath;
use dms_domain::taxonomy::{CategoryId, TagName};

pub fn library_id() -> LibraryId {
    LibraryId::new("library-1").expect("valid id")
}

pub fn added_at() -> DateTime<Utc> {
    instant("2024-01-01T10:00:00Z")
}

pub fn instant(text: &str) -> DateTime<Utc> {
    text.parse().expect("valid instant")
}

pub fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

pub fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("valid path")
}

pub fn tag(raw: &str) -> TagName {
    TagName::of(raw).expect("valid tag")
}

pub fn document_id(value: &str) -> DocumentId {
    DocumentId::new(value).expect("valid id")
}

pub fn category_id(value: &str) -> CategoryId {
    CategoryId::new(value).expect("valid id")
}

/// A valid content hash derived from `seed`, so different seeds give different hashes.
pub fn hash(seed: i32) -> ContentHash {
    ContentHash::new(format!("{seed:064x}")).expect("valid hash")
}

/// A document whose hash is derived from its id like Kotlin's `id.hashCode()`.
pub fn document(id: &str, path_text: &str) -> Document {
    document_with(id, path_text, java_hash_code(id) & i32::MAX, 100)
}

pub fn document_with(id: &str, path_text: &str, hash_seed: i32, size: u64) -> Document {
    Document::new(document_id(id), path(path_text), hash(hash_seed), size, added_at()).expect("valid document")
}

pub fn catalog_with(documents: &[Document]) -> Catalog {
    documents
        .iter()
        .fold(Catalog::empty(library_id()), |catalog, document| catalog.add_document(document.clone()).applied())
}

/// Unwraps an applied update and fails the test for a rejection.
pub trait Applied {
    fn applied(self) -> Catalog;
}

impl Applied for CatalogUpdate {
    fn applied(self) -> Catalog {
        match self {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("expected an applied update, got {reason:?}"),
        }
    }
}

impl Applied for Result<CatalogUpdate, DomainError> {
    fn applied(self) -> Catalog {
        self.expect("valid change").applied()
    }
}

/// Java's `String.hashCode()`.
fn java_hash_code(text: &str) -> i32 {
    text.encode_utf16().fold(0i32, |hash, unit| hash.wrapping_mul(31).wrapping_add(i32::from(unit)))
}

pub fn decimal(text: &str) -> dms_domain::field::Decimal {
    dms_domain::field::Decimal::parse(text).expect("valid decimal")
}

pub fn euro() -> dms_domain::field::CurrencyCode {
    dms_domain::field::CurrencyCode::new("EUR").expect("valid currency")
}

pub fn euros(amount: &str) -> dms_domain::money::Money {
    dms_domain::money::Money { amount: decimal(amount), currency: euro() }
}

pub fn contract(id: &str, name: &str) -> dms_domain::contract::Contract {
    dms_domain::contract::Contract::new(
        dms_domain::contract::ContractId::new(id).expect("valid id"),
        name,
        dms_domain::contract::ContractKind::Utility,
    )
    .expect("valid contract")
}

pub fn case(id: &str, title: &str) -> dms_domain::cases::Case {
    dms_domain::cases::Case::new(dms_domain::cases::CaseId::new(id).expect("valid id"), title, date(2026, 1, 5))
        .expect("valid case")
}
