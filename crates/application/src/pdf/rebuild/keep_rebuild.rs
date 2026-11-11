use std::sync::Arc;

use dms_domain::document::{Document, DocumentId};
use dms_domain::library::{FileNamePolicy, RelativePath};
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, SiblingNames};
use crate::library::CatalogStore;
use crate::pdf::PdfVersions;
use crate::pdf::rebuild::RebuildChoice;
use crate::port::pdf::{PdfRebuilder, RebuiltFile};
use crate::port::state::ActivityKind;
use crate::port::storage::FolderOperation;
use crate::port::system::{Clock, IdGenerator};
use crate::sync::Replicator;

/// Applies the user's choice for a rebuilt PDF. Keeping the original changes nothing. Replacing makes the rebuild a
/// new version of the document, with the scan going to the system trash. Keeping both adds the rebuild as a new
/// document next to the original, with its category, tags and fields, file first and catalog second. A scan that
/// changed after the conversion is never replaced: its newer content would go to the trash unseen
/// ([`OperationOutcome::SourceChanged`]); keeping both still works. The rebuild is deleted once it is placed, not
/// wanted, or has no document or library left to go to; after any other failure it stays, so the user can choose
/// again.
pub struct KeepRebuild {
    store: Arc<CatalogStore>,
    rebuilder: Arc<dyn PdfRebuilder>,
    versions: Arc<PdfVersions>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl KeepRebuild {
    #[allow(clippy::too_many_arguments)] // one collaborator each, as in Kotlin
    pub fn new(
        store: Arc<CatalogStore>,
        rebuilder: Arc<dyn PdfRebuilder>,
        versions: Arc<PdfVersions>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, rebuilder, versions, replicator, names, activity, clock, ids }
    }

    pub fn run(&self, document_id: &DocumentId, rebuilt: &RebuiltFile, choice: RebuildChoice) -> OperationOutcome {
        let outcome = self.apply(document_id, rebuilt, choice);
        // After these the rebuild has nowhere to go any more.
        if matches!(
            outcome,
            OperationOutcome::Done { .. } | OperationOutcome::NotFound | OperationOutcome::LibraryNotOpen
        ) {
            self.rebuilder.discard(rebuilt);
        }
        outcome
    }

    fn apply(&self, document_id: &DocumentId, rebuilt: &RebuiltFile, choice: RebuildChoice) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return OperationOutcome::NotFound;
        };
        match choice {
            RebuildChoice::KeepOriginal => {
                OperationOutcome::Done { catalog: library.catalog.clone(), pending_folders: IndexSet::new() }
            }
            RebuildChoice::ReplaceWithRebuild if document.content_hash == rebuilt.source_hash => {
                self.replace(&library, document, rebuilt)
            }
            RebuildChoice::ReplaceWithRebuild => OperationOutcome::SourceChanged,
            RebuildChoice::KeepBoth => self.add_copy(&library, document, rebuilt),
        }
    }

    fn replace(&self, library: &LibrarySnapshot, document: &Document, rebuilt: &RebuiltFile) -> OperationOutcome {
        let outcome =
            self.versions.replace(library, document, |folder, target| self.rebuilder.place(rebuilt, folder, target));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::PdfEdited, &format!("Rebuilt: {}", document.path));
        }
        outcome
    }

    fn add_copy(&self, library: &LibrarySnapshot, document: &Document, rebuilt: &RebuiltFile) -> OperationOutcome {
        let folder = document.path.parent().unwrap_or_else(RelativePath::root);
        let siblings = match self.names.of(library, &folder) {
            Ok(siblings) => siblings,
            Err(error) => return OperationOutcome::StorageFailed(error),
        };
        let siblings: Vec<&str> = siblings.iter().map(String::as_str).collect();
        let file_name = document.path.file_name();
        let base = file_name.rsplit_once('.').map_or(file_name, |(base, _)| base);
        let name =
            FileNamePolicy::next_free_name(&FileNamePolicy::sanitize(&format!("{base} (rebuilt).pdf")), &siblings);
        let path = match folder.resolve(&name) {
            Ok(path) => path,
            Err(error) => return OperationOutcome::InvalidInput(error),
        };
        let facts = match self.rebuilder.place(rebuilt, library.folders.primary(), &path) {
            Ok(facts) => facts,
            Err(error) => return OperationOutcome::StorageFailed(error),
        };
        let pending = self.replicator.distribute(
            &library.folders,
            &library.folders.sync_folders(),
            &FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: false },
        );
        let copy = Document {
            id: DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
            path,
            content_hash: facts.content_hash.expect("invariant: a written file is always hashed"),
            size: facts.size,
            added_at: self.clock.now(),
            classification: None,
            ..document.clone()
        };
        let outcome = self.store.commit(|catalog| catalog.add_document(copy.clone())).with_pending_folders(&pending);
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::PdfEdited, &format!("Rebuilt as {}", copy.path));
        }
        outcome
    }
}
