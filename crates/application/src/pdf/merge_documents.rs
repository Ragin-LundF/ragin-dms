use std::sync::Arc;

use dms_domain::document::{Document, DocumentId, DocumentKind, DocumentMetadata};
use dms_domain::library::{FileNamePolicy, RelativePath};
use dms_domain::text;

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, SiblingNames};
use crate::document::TrashDocument;
use crate::library::CatalogStore;
use crate::port::pdf::PdfEditor;
use crate::port::state::ActivityKind;
use crate::port::storage::FolderOperation;
use crate::port::system::{Clock, IdGenerator};
use crate::sync::Replicator;

/// Merges PDF documents into a new one named `file_name`, in the given order, in the folder of the first. The new
/// document takes over category, tags and correspondent of the first. The sources go to the trash only with
/// `trash_sources`, which the user confirmed.
pub struct MergeDocuments {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    trash_document: Arc<TrashDocument>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl MergeDocuments {
    #[allow(clippy::too_many_arguments)] // one collaborator each, as in Kotlin
    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        trash_document: Arc<TrashDocument>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, pdf, replicator, names, trash_document, activity, clock, ids }
    }

    pub fn run(&self, document_ids: &[DocumentId], file_name: &str, trash_sources: bool) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(sources) =
            document_ids.iter().map(|id| library.catalog.documents().get(id)).collect::<Option<Vec<&Document>>>()
        else {
            return OperationOutcome::NotFound;
        };
        if sources.len() < 2 || sources.iter().any(|source| source.kind() != DocumentKind::Pdf) {
            return OperationOutcome::NotFound;
        }
        let first = sources[0];
        let path = match self.target_of(&library, first, file_name) {
            Ok(path) => path,
            Err(outcome) => return outcome,
        };
        let source_paths: Vec<RelativePath> = sources.iter().map(|source| source.path.clone()).collect();
        let facts = match self.pdf.merge(library.folders.primary(), &source_paths, &path) {
            Ok(facts) => facts,
            Err(error) => return OperationOutcome::StorageFailed(error),
        };
        let pending = self.replicator.distribute(
            &library.folders,
            &library.folders.sync_folders(),
            &FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: false },
        );
        let merged = Document {
            metadata: DocumentMetadata { title: None, ..first.metadata.clone() },
            category_id: first.category_id.clone(),
            tags: first.tags.clone(),
            ..Document::new(
                DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
                path.clone(),
                facts.content_hash.expect("invariant: a written file is always hashed"),
                facts.size,
                self.clock.now(),
            )
            .expect("invariant: a merged file is never the root")
        };
        let outcome = self.store.commit(|catalog| catalog.add_document(merged.clone())).with_pending_folders(&pending);
        let OperationOutcome::Done { pending_folders, .. } = &outcome else {
            return outcome;
        };
        self.activity.record(ActivityKind::PdfEdited, &format!("Merged {} documents into {path}", sources.len()));
        if trash_sources {
            let pending = pending_folders.clone();
            self.trash(&sources, outcome, pending)
        } else {
            outcome
        }
    }

    /// A free path for the merged file next to `first`, or why there is none.
    #[allow(clippy::result_large_err)] // the refusal is the outcome of the whole action, built once
    #[allow(clippy::case_sensitive_file_extension_comparisons)] // lowercased first, like Kotlin's ignoreCase
    fn target_of(
        &self,
        library: &LibrarySnapshot,
        first: &Document,
        file_name: &str,
    ) -> Result<RelativePath, OperationOutcome> {
        let name = if text::lowercase(file_name).ends_with(".pdf") {
            file_name.to_owned()
        } else {
            format!("{file_name}.pdf")
        };
        if let Some(violation) = FileNamePolicy::check(&name) {
            return Err(OperationOutcome::InvalidName(violation));
        }
        let folder = first.path.parent().unwrap_or_else(RelativePath::root);
        let siblings = self.names.of(library, &folder).map_err(OperationOutcome::StorageFailed)?;
        let siblings: Vec<&str> = siblings.iter().map(String::as_str).collect();
        folder.resolve(&FileNamePolicy::next_free_name(&name, &siblings)).map_err(OperationOutcome::InvalidInput)
    }

    /// Moves the sources to the trash one by one; the first refusal stops and is returned.
    fn trash(
        &self,
        sources: &[&Document],
        merged: OperationOutcome,
        mut pending: indexmap::IndexSet<dms_domain::library::LibraryFolderId>,
    ) -> OperationOutcome {
        let mut last = merged;
        for source in sources {
            last = self.trash_document.run(&source.id, true);
            let OperationOutcome::Done { pending_folders, .. } = &last else {
                return last;
            };
            pending.extend(pending_folders.iter().cloned());
        }
        last.with_pending_folders(&pending)
    }
}
