use std::sync::Arc;

use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId, DocumentMetadata};
use dms_domain::library::{FileNamePolicy, RelativePath};

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, SiblingNames};
use crate::emergency::{EmergencyOutcome, OverviewTarget, emergency_overview};
use crate::library::CatalogStore;
use crate::port::export::ExportWriter;
use crate::port::pdf::{PdfEditor, TextDocument};
use crate::port::state::ActivityKind;
use crate::port::storage::FolderOperation;
use crate::port::system::{Clock, IdGenerator};
use crate::sync::Replicator;

const TITLE: &str = "Emergency overview";

/// Saves the overview of the emergency folder as a PDF: into a library folder (every library folder, then
/// catalogued as "Emergency overview" dated today), or into a place outside the library the user chose.
pub struct CreateEmergencyOverview {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    exports: Arc<dyn ExportWriter>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl CreateEmergencyOverview {
    #[allow(clippy::too_many_arguments)] // the ports and collaborators of one use case, wired once
    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        exports: Arc<dyn ExportWriter>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, pdf, exports, replicator, names, activity, clock, ids }
    }

    pub fn run(&self, target: &OverviewTarget) -> EmergencyOutcome {
        let Some(library) = self.store.current() else {
            return EmergencyOutcome::Refused(OperationOutcome::LibraryNotOpen);
        };
        let settings = library.catalog.settings();
        let Some(folder) = settings.emergency_folder.as_ref() else {
            return EmergencyOutcome::NothingIncluded;
        };
        let today = self.clock.today();
        let document = emergency_overview(folder, &library.catalog, settings.owner.as_ref(), today);
        let outcome = match target {
            OverviewTarget::Library { folder } => self.save_in_library(&library, folder, &document),
            OverviewTarget::Outside { directory, name } => self.save_outside(&library, directory, name, &document),
        };
        if let EmergencyOutcome::Saved { location, .. } = &outcome {
            self.activity.record(ActivityKind::DocumentAdded, &format!("Saved the emergency overview {location}"));
        }
        outcome
    }

    fn save_outside(
        &self,
        library: &LibrarySnapshot,
        directory: &str,
        name: &str,
        document: &TextDocument,
    ) -> EmergencyOutcome {
        if self.exports.is_inside(directory, library.folders.folders()) {
            return EmergencyOutcome::InsideLibrary;
        }
        let written =
            self.pdf.text_document_bytes(document).and_then(|bytes| self.exports.write_file(directory, name, &bytes));
        match written {
            Ok(location) => EmergencyOutcome::Saved { location, document_id: None, documents: 0 },
            Err(error) => EmergencyOutcome::Refused(OperationOutcome::StorageFailed(error)),
        }
    }

    fn save_in_library(
        &self,
        library: &LibrarySnapshot,
        folder: &RelativePath,
        document: &TextDocument,
    ) -> EmergencyOutcome {
        let today = self.clock.today();
        let path = match self.free_path(library, folder, &format!("{TITLE} {today}.pdf")) {
            Ok(path) => path,
            Err(outcome) => return EmergencyOutcome::Refused(outcome),
        };
        let facts = match self.pdf.write_text_document(library.folders.primary(), &path, document) {
            Ok(facts) => facts,
            Err(error) => return EmergencyOutcome::Refused(OperationOutcome::StorageFailed(error)),
        };
        let pending = self.replicator.distribute(
            &library.folders,
            &library.folders.sync_folders(),
            &FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: false },
        );
        let id = DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank");
        let overview = Document {
            metadata: DocumentMetadata {
                title: Some(TITLE.to_owned()),
                document_date: Some(today),
                correspondent: None,
            },
            ..Document::new(
                id.clone(),
                path.clone(),
                facts.content_hash.expect("invariant: a written file is always hashed"),
                facts.size,
                self.clock.now(),
            )
            .expect("invariant: an overview file is never the root")
        };
        let outcome = self
            .store
            .commit(|catalog: &Catalog| catalog.add_document(overview.clone()))
            .with_pending_folders(&pending);
        match outcome {
            OperationOutcome::Done { .. } => {
                EmergencyOutcome::Saved { location: path.value().to_owned(), document_id: Some(id), documents: 0 }
            }
            other => EmergencyOutcome::Refused(other),
        }
    }

    #[allow(clippy::result_large_err)] // the refusal is the outcome of the whole action, built once
    fn free_path(
        &self,
        library: &LibrarySnapshot,
        folder: &RelativePath,
        name: &str,
    ) -> Result<RelativePath, OperationOutcome> {
        let siblings = self.names.of(library, folder).map_err(OperationOutcome::StorageFailed)?;
        let siblings: Vec<&str> = siblings.iter().map(String::as_str).collect();
        folder
            .resolve(&FileNamePolicy::next_free_name(&FileNamePolicy::sanitize(name), &siblings))
            .map_err(OperationOutcome::InvalidInput)
    }
}
