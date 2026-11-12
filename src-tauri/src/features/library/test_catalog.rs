//! Synthetic catalogs for the presenter tests. Invented names only.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::document::{ContentHash, Document, DocumentId};
use dms_domain::field::{CurrencyCode, Decimal, FieldKey, FieldValue};
use dms_domain::library::RelativePath;
use dms_domain::money::Money;
use dms_domain::settings::CatalogSettings;
use dms_domain::tax::{TaxArea, TaxAssignment};
use dms_domain::taxonomy::{Category, CategoryId, TagName};

use super::model::Moment;

pub fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("valid path")
}

pub fn tag(raw: &str) -> TagName {
    TagName::of(raw).expect("valid tag")
}

pub fn category_id(raw: &str) -> CategoryId {
    CategoryId::new(raw).expect("valid id")
}

pub fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

pub fn instant(text: &str) -> DateTime<Utc> {
    text.parse().expect("valid instant")
}

/// 2025-10-15 noon in Berlin.
pub fn moment() -> Moment {
    Moment { now: instant("2025-10-15T10:00:00Z"), zone: Tz::Europe__Berlin }
}

/// Builds a catalog step by step: categories first, then documents.
pub struct TestCatalog {
    catalog: Catalog,
    next: usize,
}

impl TestCatalog {
    pub fn new() -> Self {
        Self { catalog: Catalog::empty(LibraryId::new("library-1").expect("valid id")), next: 0 }
    }

    pub fn category(mut self, id: &str, name: &str, parent: Option<&str>) -> Self {
        let category = Category::new(category_id(id), name, parent.map(category_id), None).expect("valid category");
        self.catalog = applied(self.catalog.add_category(category).expect("valid change"));
        self
    }

    pub fn document(mut self, path_text: &str, change: impl FnOnce(&mut DocumentBuilder)) -> Self {
        self.next += 1;
        let id = format!("d-{:03}", self.next);
        let hash = ContentHash::new(format!("{:064x}", self.next)).expect("valid hash");
        let document = Document::new(
            DocumentId::new(id).expect("valid id"),
            path(path_text),
            hash,
            100,
            instant("2024-01-01T10:00:00Z"),
        )
        .expect("valid document");
        let mut builder = DocumentBuilder { document };
        change(&mut builder);
        self.catalog = applied(self.catalog.add_document(builder.document).expect("valid change"));
        self
    }

    pub fn settings(mut self, change: impl FnOnce(&mut CatalogSettings)) -> Self {
        let mut settings = self.catalog.settings().clone();
        change(&mut settings);
        self.catalog = applied(self.catalog.change_settings(settings).expect("valid settings"));
        self
    }

    pub fn build(self) -> Catalog {
        self.catalog
    }
}

pub struct DocumentBuilder {
    document: Document,
}

impl DocumentBuilder {
    pub fn title(&mut self, title: &str) -> &mut Self {
        self.document.metadata.title = Some(title.to_owned());
        self
    }

    pub fn correspondent(&mut self, name: &str) -> &mut Self {
        self.document.metadata.correspondent = Some(name.to_owned());
        self
    }

    pub fn dated(&mut self, date: NaiveDate) -> &mut Self {
        self.document.metadata.document_date = Some(date);
        self
    }

    pub fn added(&mut self, at: &str) -> &mut Self {
        self.document.added_at = instant(at);
        self
    }

    pub fn category(&mut self, id: &str) -> &mut Self {
        self.document.category_id = Some(category_id(id));
        self
    }

    pub fn tags(&mut self, tags: &[&str]) -> &mut Self {
        for raw in tags {
            self.document.tags.insert(tag(raw));
        }
        self
    }

    pub fn money(&mut self, key: &str, amount: &str) -> &mut Self {
        let value = FieldValue::Money {
            amount: Decimal::parse(amount).expect("valid amount"),
            currency: CurrencyCode::new("EUR").expect("valid currency"),
        };
        self.document.fields.insert(FieldKey::new(key).expect("valid key"), value);
        self
    }

    /// Assigns the document to a tax year and area, with a deductible part in euros if given.
    pub fn tax(&mut self, year: i32, area: TaxArea, deductible: Option<&str>) -> &mut Self {
        let deductible = deductible.map(|amount| Money {
            amount: Decimal::parse(amount).expect("valid amount"),
            currency: CurrencyCode::new("EUR").expect("valid currency"),
        });
        self.document.tax = Some(TaxAssignment { year, area, deductible });
        self
    }

    pub fn field_date(&mut self, key: &str, date: NaiveDate) -> &mut Self {
        self.document.fields.insert(FieldKey::new(key).expect("valid key"), FieldValue::Date(date));
        self
    }
}

fn applied(update: CatalogUpdate) -> Catalog {
    match update {
        CatalogUpdate::Applied(catalog) => catalog,
        CatalogUpdate::Rejected(reason) => panic!("expected an applied update, got {reason:?}"),
    }
}
