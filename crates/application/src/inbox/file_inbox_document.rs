use std::sync::Arc;

use dms_domain::catalog::CatalogRejection;
use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;

use crate::common::{ActivityRecorder, DocumentFiling, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Files an inbox document by hand, without any AI: one move into the target folder in every library folder,
/// optionally with a new name. A missing target folder is created; a taken name gets a ` (2)` suffix. Once the
/// document left the inbox, it is an ordinary document.
pub struct FileInboxDocument {
    store: Arc<CatalogStore>,
    filing: Arc<DocumentFiling>,
    activity: Arc<ActivityRecorder>,
}

impl FileInboxDocument {
    pub fn new(store: Arc<CatalogStore>, filing: Arc<DocumentFiling>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, filing, activity }
    }

    pub fn run(
        &self,
        document_id: &DocumentId,
        target_folder: &RelativePath,
        new_name: Option<&str>,
    ) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let inbox = &library.catalog.settings().layout.inbox;
        if inbox.enabled && (*target_folder == inbox.path || target_folder.is_inside(&inbox.path)) {
            return OperationOutcome::Rejected(CatalogRejection::InvalidFolderMove);
        }
        let outcome = self.filing.file(document_id, target_folder, new_name);
        if let OperationOutcome::Done { catalog, .. } = &outcome {
            let path = catalog.documents().get(document_id).map_or(target_folder, |document| &document.path);
            self.activity.record(ActivityKind::DocumentFiled, path.value());
        }
        outcome
    }
}
