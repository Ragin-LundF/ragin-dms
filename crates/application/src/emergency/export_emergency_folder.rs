use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::document::Document;
use dms_domain::emergency::EmergencyEntry;
use dms_domain::library::FileNamePolicy;
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::emergency::emergency_overview::included_in_order;
use crate::emergency::{EmergencyOutcome, emergency_overview};
use crate::export::{ExportOptions, ExportPlanner};
use crate::library::CatalogStore;
use crate::port::export::{ExportTarget, ExportWriter};
use crate::port::pdf::PdfEditor;
use crate::port::state::ActivityKind;
use crate::port::system::Clock;

const OVERVIEW_NAME: &str = "Emergency overview.pdf";
const MIN_PASSWORD_LENGTH: usize = 8;

/// Writes the emergency folder for a trusted person into a ZIP file outside the library, every entry encrypted with
/// AES-256: the overview and the included documents, together with the documents of the included contracts. The
/// password is passed through to the writer only; it is never stored or logged.
pub struct ExportEmergencyFolder {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    exports: Arc<dyn ExportWriter>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
}

impl ExportEmergencyFolder {
    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        exports: Arc<dyn ExportWriter>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { store, pdf, exports, activity, clock }
    }

    pub fn run(&self, directory: &str, name: &str, password: &str) -> EmergencyOutcome {
        let Some(library) = self.store.current() else {
            return EmergencyOutcome::Refused(OperationOutcome::LibraryNotOpen);
        };
        if password.chars().count() < MIN_PASSWORD_LENGTH {
            let error = DomainError::InvalidArgument("The password needs at least 8 characters".to_owned());
            return EmergencyOutcome::Refused(OperationOutcome::InvalidInput(error));
        }
        let target = match ExportTarget::new(directory, name, true) {
            Ok(target) => target,
            Err(error) => return EmergencyOutcome::Refused(OperationOutcome::InvalidInput(error)),
        };
        if self.exports.is_inside(directory, library.folders.folders()) {
            return EmergencyOutcome::InsideLibrary;
        }
        let catalog = &library.catalog;
        let Some(folder) =
            catalog.settings().emergency_folder.as_ref().filter(|it| it.items.iter().any(|i| i.included))
        else {
            return EmergencyOutcome::NothingIncluded;
        };
        let mut ids = IndexSet::new();
        for item in included_in_order(folder) {
            match &item.entry {
                EmergencyEntry::Document(id) => {
                    ids.insert(id.clone());
                }
                EmergencyEntry::Contract(id) => {
                    ids.extend(catalog.contracts().get(id).into_iter().flat_map(|it| it.document_ids.iter().cloned()));
                }
                EmergencyEntry::Contact { .. } | EmergencyEntry::Note(_) => {}
            }
        }
        let documents: Vec<Document> = ids.iter().filter_map(|id| catalog.documents().get(id)).cloned().collect();
        let options = ExportOptions { flatten: true, rename_to_title: false, index: false, zip: true, tax_year: None };
        let bundle = ExportPlanner::plan(catalog, &documents, options, self.clock.zone());
        let overview = emergency_overview(folder, catalog, catalog.settings().owner.as_ref(), self.clock.today());
        let taken: Vec<&str> = bundle.entries.iter().map(|entry| entry.target.as_str()).collect();
        let overview_name = FileNamePolicy::next_free_name(OVERVIEW_NAME, &taken);
        let written = self.pdf.text_document_bytes(&overview).and_then(|bytes| {
            self.exports.write_encrypted_zip(
                library.folders.primary(),
                &bundle,
                &[(overview_name, bytes)],
                &target,
                password,
            )
        });
        match written {
            Ok(location) => {
                self.activity.record(ActivityKind::Exported, &format!("The emergency folder to {location}"));
                EmergencyOutcome::Saved { location, document_id: None, documents: documents.len() }
            }
            Err(error) => EmergencyOutcome::Refused(OperationOutcome::StorageFailed(error)),
        }
    }
}
