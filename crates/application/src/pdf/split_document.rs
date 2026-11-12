use std::ops::RangeInclusive;
use std::sync::Arc;

use dms_domain::catalog::CatalogUpdate;
use dms_domain::document::{Document, DocumentId, DocumentKind, DocumentMetadata};
use dms_domain::library::{FileNamePolicy, LibraryFolderId, RelativePath};
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, SiblingNames};
use crate::document::TrashDocument;
use crate::library::CatalogStore;
use crate::port::pdf::{PageChange, PdfEditor};
use crate::port::state::ActivityKind;
use crate::port::storage::FolderOperation;
use crate::port::system::{Clock, IdGenerator};
use crate::sync::Replicator;

/// Splits a PDF into new documents, one per page range (pages counted from 0), next to the original. Each new
/// document takes over category, tags and correspondent. The original goes to the trash only with `trash_original`,
/// which the user confirmed.
pub struct SplitDocument {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    trash_document: Arc<TrashDocument>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl SplitDocument {
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

    pub fn run(
        &self,
        document_id: &DocumentId,
        ranges: &[RangeInclusive<usize>],
        trash_original: bool,
    ) -> OperationOutcome {
        self.split(document_id, ranges, trash_original).0
    }

    /// Like [`Self::run`], with the ids of the new documents in the order of `ranges`; none unless they were
    /// catalogued.
    pub fn split(
        &self,
        document_id: &DocumentId,
        ranges: &[RangeInclusive<usize>],
        trash_original: bool,
    ) -> (OperationOutcome, Vec<DocumentId>) {
        let Some(library) = self.store.current() else {
            return (OperationOutcome::LibraryNotOpen, Vec::new());
        };
        let Some(original) = library.catalog.documents().get(document_id) else {
            return (OperationOutcome::NotFound, Vec::new());
        };
        // Negative starts, which Kotlin also refused, cannot be expressed with `usize`.
        let valid_ranges = !ranges.is_empty() && !ranges.iter().any(RangeInclusive::is_empty);
        if original.kind() != DocumentKind::Pdf || !valid_ranges {
            return (OperationOutcome::NotFound, Vec::new());
        }
        let mut created: Vec<Document> = Vec::new();
        let mut pending: IndexSet<LibraryFolderId> = IndexSet::new();
        for range in ranges {
            let taken: Vec<String> = created.iter().map(|document| document.path.file_name().to_owned()).collect();
            match self.write_part(&library, original, range, &taken) {
                Err(outcome) => return (outcome, Vec::new()),
                Ok((document, part_pending)) => {
                    created.push(document);
                    pending.extend(part_pending);
                }
            }
        }
        let outcome = self
            .store
            .commit(|catalog| {
                created.iter().try_fold(CatalogUpdate::Applied(catalog.clone()), |update, document| match update {
                    CatalogUpdate::Applied(catalog) => catalog.add_document(document.clone()),
                    rejected @ CatalogUpdate::Rejected(_) => Ok(rejected),
                })
            })
            .with_pending_folders(&pending);
        if !matches!(outcome, OperationOutcome::Done { .. }) {
            return (outcome, Vec::new());
        }
        self.activity
            .record(ActivityKind::PdfEdited, &format!("Split {} into {} documents", original.path, created.len()));
        let ids = created.into_iter().map(|document| document.id).collect();
        if trash_original { (self.trash_document.run(&original.id, true), ids) } else { (outcome, ids) }
    }

    #[allow(clippy::result_large_err)] // the refusal is the outcome of the whole action, built once
    fn write_part(
        &self,
        library: &LibrarySnapshot,
        original: &Document,
        range: &RangeInclusive<usize>,
        taken: &[String],
    ) -> Result<(Document, IndexSet<LibraryFolderId>), OperationOutcome> {
        let folder = original.path.parent().unwrap_or_else(RelativePath::root);
        let siblings = self.names.of(library, &folder).map_err(OperationOutcome::StorageFailed)?;
        let file_name = original.path.file_name();
        let base = file_name.rsplit_once('.').map_or(file_name, |(base, _)| base);
        let label = if range.start() == range.end() {
            format!("page {}", range.start() + 1)
        } else {
            format!("pages {}-{}", range.start() + 1, range.end() + 1)
        };
        let all_taken: Vec<&str> =
            siblings.iter().map(String::as_str).chain(taken.iter().map(String::as_str)).collect();
        let name =
            FileNamePolicy::next_free_name(&FileNamePolicy::sanitize(&format!("{base} ({label}).pdf")), &all_taken);
        let path = folder.resolve(&name).map_err(OperationOutcome::InvalidInput)?;
        let pages: Vec<PageChange> = range.clone().map(|page| PageChange { page, rotation: 0 }).collect();
        let facts = self
            .pdf
            .rewrite(library.folders.primary(), &original.path, &pages, &path)
            .map_err(OperationOutcome::StorageFailed)?;
        let pending = self.replicator.distribute(
            &library.folders,
            &library.folders.sync_folders(),
            &FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: false },
        );
        let document = Document {
            metadata: DocumentMetadata {
                title: original.metadata.title.as_ref().map(|title| format!("{title} ({label})")),
                ..original.metadata.clone()
            },
            category_id: original.category_id.clone(),
            tags: original.tags.clone(),
            ..Document::new(
                DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
                path,
                facts.content_hash.expect("invariant: a written file is always hashed"),
                facts.size,
                self.clock.now(),
            )
            .expect("invariant: a part is never the root")
        };
        Ok((document, pending))
    }
}
