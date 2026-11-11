use std::sync::Arc;

use dms_domain::document::{Document, DocumentId, DocumentMetadata, FileNameDates};
use dms_domain::library::{FileNamePolicy, RelativePath};

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, SiblingNames};
use crate::document::{AddDocumentsOutcome, AddedFile};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, ExternalFile, FolderOperation, StorageError, StorageFailure};
use crate::port::system::{Clock, IdGenerator};
use crate::sync::{Replication, Replicator};

/// Copies files from outside the library into a target folder of every library folder and catalogs them. The
/// original name is kept; a taken name gets a ` (2)` suffix. Files with content the library already has are added
/// as well and reported as duplicates.
pub struct AddDocuments {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl AddDocuments {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, storage, replicator, names, activity, clock, ids }
    }

    pub fn run(&self, files: &[ExternalFile], target_folder: &RelativePath) -> AddDocumentsOutcome {
        if self.store.current().is_none() {
            return AddDocumentsOutcome::LibraryNotOpen;
        }
        AddDocumentsOutcome::Added(files.iter().map(|file| self.add(file, target_folder)).collect())
    }

    fn add(&self, file: &ExternalFile, target_folder: &RelativePath) -> AddedFile {
        let Some(library) = self.store.current() else {
            return AddedFile::failed(file.clone(), OperationOutcome::LibraryNotOpen);
        };
        if let Some(violation) = FileNamePolicy::check(file.file_name()) {
            return AddedFile::failed(file.clone(), OperationOutcome::InvalidName(violation));
        }
        let siblings = match self.names.of(&library, target_folder) {
            Ok(siblings) => siblings,
            Err(error) => return AddedFile::failed(file.clone(), OperationOutcome::StorageFailed(error)),
        };
        let siblings: Vec<&str> = siblings.iter().map(String::as_str).collect();
        let free_name = FileNamePolicy::next_free_name(file.file_name(), &siblings);
        let target = match target_folder.resolve(&free_name) {
            Ok(target) => target,
            Err(error) => return AddedFile::failed(file.clone(), OperationOutcome::InvalidInput(error)),
        };
        let outcome = self.copy_and_catalog(&library, file, &target);
        let OperationOutcome::Done { catalog, .. } = &outcome else {
            return AddedFile::failed(file.clone(), outcome);
        };
        self.activity.record(ActivityKind::DocumentAdded, target.value());
        let duplicates = catalog
            .find_by_path(&target)
            .map(|document| {
                catalog
                    .find_by_content_hash(&document.content_hash)
                    .into_iter()
                    .filter(|other| other.id != document.id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        AddedFile { source: file.clone(), outcome, path: Some(target), duplicates }
    }

    /// Writes the file to every folder first and the catalog second.
    fn copy_and_catalog(
        &self,
        library: &LibrarySnapshot,
        file: &ExternalFile,
        target: &RelativePath,
    ) -> OperationOutcome {
        let replication = self.replicator.replicate(
            &library.folders,
            |primary| self.storage.import_file(primary, file, target),
            &FolderOperation::CopyFromPrimary { path: target.clone(), replace_existing: false },
        );
        let (facts, pending) = match replication {
            Replication::PrimaryFailed(error) => return OperationOutcome::StorageFailed(error),
            Replication::Done { value, pending_folders } => (value, pending_folders),
        };
        let Some(content_hash) = facts.content_hash else {
            return OperationOutcome::StorageFailed(StorageError::new(StorageFailure::IoError));
        };
        let document = Document {
            // The copy was just created, so only the name can tell how old the document is.
            metadata: DocumentMetadata {
                document_date: FileNameDates::date_in(target.file_name()),
                ..DocumentMetadata::default()
            },
            ..Document::new(
                DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
                target.clone(),
                content_hash,
                facts.size,
                self.clock.now(),
            )
            .expect("invariant: a file in a folder is never the root")
        };
        self.store.commit(|catalog| catalog.add_document(document.clone())).with_pending_folders(&pending)
    }
}
