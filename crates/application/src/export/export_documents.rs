use std::sync::Arc;

use dms_domain::catalog::Catalog;
use dms_domain::document::Document;

use crate::common::ActivityRecorder;
use crate::export::missing_items::{missing_items_file, missing_items_of, missing_items_page};
use crate::export::{ExportOptions, ExportOutcome, ExportPlanner, ExportScope};
use crate::library::CatalogStore;
use crate::port::export::{ExportBundle, ExportTarget, ExportWriter};
use crate::port::pdf::PdfEditor;
use crate::port::state::ActivityKind;
use crate::port::storage::StorageResult;
use crate::port::system::Clock;
use crate::search::ContentMatcher;

/// Exports documents into a folder or a ZIP file outside the library, with an index, for example for the tax
/// adviser. It only reads the library; it never writes into a library folder.
pub struct ExportDocuments {
    store: Arc<CatalogStore>,
    search: Arc<ContentMatcher>,
    writer: Arc<dyn ExportWriter>,
    pdf: Arc<dyn PdfEditor>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
}

impl ExportDocuments {
    pub fn new(
        store: Arc<CatalogStore>,
        search: Arc<ContentMatcher>,
        writer: Arc<dyn ExportWriter>,
        pdf: Arc<dyn PdfEditor>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { store, search, writer, pdf, activity, clock }
    }

    pub fn run(&self, scope: &ExportScope, directory: &str, name: &str, options: ExportOptions) -> ExportOutcome {
        let Some(library) = self.store.current() else {
            return ExportOutcome::LibraryNotOpen;
        };
        if self.writer.is_inside(directory, library.folders.folders()) {
            return ExportOutcome::InsideLibrary;
        }
        let documents: Vec<Document> = match scope {
            ExportScope::Documents(ids) => {
                ids.iter().filter_map(|id| library.catalog.documents().get(id)).cloned().collect()
            }
            ExportScope::Query(query) => self.search.matching(&library, query),
        };
        if documents.is_empty() {
            return ExportOutcome::Empty;
        }
        let mut bundle = ExportPlanner::plan(&library.catalog, &documents, options, self.clock.zone());
        if let Some(year) = options.tax_year
            && let Err(error) = self.add_missing_items(&mut bundle, &library.catalog, year)
        {
            return ExportOutcome::Failed(error);
        }
        let target = match ExportTarget::new(directory, name, options.zip) {
            Ok(target) => target,
            Err(error) => return ExportOutcome::InvalidInput(error),
        };
        match self.writer.write(library.folders.primary(), &bundle, &target) {
            Err(error) => ExportOutcome::Failed(error),
            Ok(location) => {
                self.activity.record(ActivityKind::Exported, &format!("{} documents to {location}", documents.len()));
                ExportOutcome::Exported { count: documents.len(), location }
            }
        }
    }

    fn add_missing_items(&self, bundle: &mut ExportBundle, catalog: &Catalog, year: i32) -> StorageResult<()> {
        let today = self.clock.today();
        let items = missing_items_of(catalog, year, today);
        let page = self.pdf.text_document_bytes(&missing_items_page(year, today, &items))?;
        bundle.files.insert(0, missing_items_file(page));
        bundle.missing_items = Some(items);
        Ok(())
    }
}
