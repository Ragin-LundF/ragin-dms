//! Synthetic catalogs for the presenter tests; invented names only.

use chrono::{NaiveDate, TimeZone, Utc};
use dms_domain::catalog::{Catalog, CatalogRevision, LibraryId};
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{ContentHash, Document, DocumentId};
use dms_domain::field::{FieldPresets, FieldValue, FieldValues};
use dms_domain::library::RelativePath;
use dms_domain::link::DocumentLink;
use dms_domain::parse_iso_date;
use dms_domain::settings::CatalogSettings;
use dms_domain::taxonomy::{Category, CategoryId, TagName};

pub fn day(text: &str) -> NaiveDate {
    parse_iso_date(text).expect("test date")
}

pub fn id(value: &str) -> DocumentId {
    DocumentId::new(value).expect("test id")
}

pub fn category_id(value: &str) -> CategoryId {
    CategoryId::new(value).expect("test category id")
}

/// A document added on 2026-01-`index`, with a content hash of its own.
pub fn document(index: u32, path: &str) -> Document {
    let added_at = Utc.with_ymd_and_hms(2026, 1, index, 9, 0, 0).single().expect("test instant");
    Document::new(
        id(&format!("d{index}")),
        RelativePath::of(path).expect("test path"),
        ContentHash::new(format!("{index:064x}")).expect("test hash"),
        1_000,
        added_at,
    )
    .expect("test document")
}

pub fn tag(value: &str) -> TagName {
    TagName::of(value).expect("test tag")
}

pub fn with_tags(mut document: Document, tags: &[&str]) -> Document {
    document.tags = tags.iter().map(|value| tag(value)).collect();
    document
}

pub fn category(id: &str, name: &str, parent: Option<&str>) -> Category {
    Category::new(category_id(id), name, parent.map(category_id), None).expect("test category")
}

/// The standard fields (amount, due, cancel by, contract end, warranty until), EUR as default currency.
pub fn standard_settings() -> CatalogSettings {
    let mut settings = CatalogSettings { field_definitions: FieldPresets::all(), ..CatalogSettings::default() };
    settings.layout.default_currency = dms_domain::field::CurrencyCode::parse("EUR");
    settings
}

pub fn field_value(document: &mut Document, key: &str, raw: &str, settings: &CatalogSettings) {
    let key = dms_domain::field::FieldKey::new(key).expect("test key");
    let definition = settings.field(&key).expect("test field defined");
    let value: FieldValue =
        FieldValues::parse(definition.field_type, raw, settings.layout.default_currency.as_ref()).expect("test value");
    document.fields.insert(key, value);
}

pub fn link(first: &str, second: &str, label: Option<&str>) -> DocumentLink {
    DocumentLink::new(id(first), id(second), label).expect("test link")
}

/// A catalog of `documents`, `categories` and `links`; its tags are the documents' tags.
pub fn catalog(
    documents: Vec<Document>,
    categories: Vec<Category>,
    links: Vec<DocumentLink>,
    settings: CatalogSettings,
) -> Catalog {
    let tags = documents.iter().flat_map(|document| document.tags.iter().cloned()).collect();
    Catalog::new(
        LibraryId::new("test-library").expect("test library id"),
        CatalogRevision::INITIAL,
        documents.into_iter().map(|document| (document.id.clone(), document)).collect(),
        categories.into_iter().map(|category| (category.id.clone(), category)).collect(),
        tags,
        links.into_iter().collect(),
        settings,
    )
    .expect("test catalog")
}

pub fn suggestion(document_id: &str) -> ClassificationSuggestion {
    ClassificationSuggestion::empty(id(document_id), "test-model")
}
