use std::sync::Arc;

use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId, DocumentKind};
use indexmap::IndexSet;

use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::pdf::PdfVersions;
use crate::port::pdf::{PdfEditor, PdfMetadata};

/// Writes the catalog's title, correspondent (author), category (subject), tags (keywords) and date into a PDF, as
/// a new version, so other applications show them. Only while the library's setting for it is on; it is off by
/// default.
pub struct EmbedPdfMetadata {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    versions: Arc<PdfVersions>,
}

impl EmbedPdfMetadata {
    pub fn new(store: Arc<CatalogStore>, pdf: Arc<dyn PdfEditor>, versions: Arc<PdfVersions>) -> Self {
        Self { store, pdf, versions }
    }

    pub fn run(&self, document_id: &DocumentId) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return OperationOutcome::NotFound;
        };
        if document.kind() != DocumentKind::Pdf || !library.catalog.settings().layout.embed_metadata {
            return OperationOutcome::Done { catalog: library.catalog.clone(), pending_folders: IndexSet::new() };
        }
        let metadata = metadata_of(document, &library.catalog);
        self.versions.replace(&library, document, |folder, target| {
            self.pdf.write_metadata(folder, &document.path, &metadata, target)
        })
    }
}

fn metadata_of(document: &Document, catalog: &Catalog) -> PdfMetadata {
    let categories = catalog.categories();
    // Bounded by the number of categories, like Kotlin's `take(n)`, so a cycle cannot loop forever.
    let mut names: Vec<&str> =
        std::iter::successors(document.category_id.as_ref().and_then(|id| categories.get(id)), |current| {
            current.parent_id.as_ref().and_then(|id| categories.get(id))
        })
        .take(categories.len())
        .map(|category| category.name.as_str())
        .collect();
    names.reverse();
    let mut tags: Vec<_> = document.tags.iter().collect();
    tags.sort();
    PdfMetadata {
        title: document.metadata.title.clone(),
        author: document.metadata.correspondent.clone(),
        subject: (!names.is_empty()).then(|| names.join(" / ")),
        keywords: tags.into_iter().map(|tag| tag.value().to_owned()).collect(),
        date: document.metadata.document_date,
    }
}
