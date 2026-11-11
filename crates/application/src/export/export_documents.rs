use std::sync::Arc;

use dms_domain::document::Document;

use crate::common::ActivityRecorder;
use crate::export::{ExportOptions, ExportOutcome, ExportPlanner, ExportScope};
use crate::library::CatalogStore;
use crate::port::export::{ExportTarget, ExportWriter};
use crate::port::state::ActivityKind;
use crate::port::system::Clock;
use crate::search::ContentMatcher;

/// Exports documents into a folder or a ZIP file outside the library, with an index, for example for the tax
/// adviser. It only reads the library; it never writes into a library folder.
pub struct ExportDocuments {
    store: Arc<CatalogStore>,
    search: Arc<ContentMatcher>,
    writer: Arc<dyn ExportWriter>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
}

impl ExportDocuments {
    pub fn new(
        store: Arc<CatalogStore>,
        search: Arc<ContentMatcher>,
        writer: Arc<dyn ExportWriter>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { store, search, writer, activity, clock }
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
        let bundle = ExportPlanner::plan(&library.catalog, &documents, options, self.clock.zone());
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
}
