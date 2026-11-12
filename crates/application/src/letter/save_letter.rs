use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogUpdate};
use dms_domain::document::{Document, DocumentId, DocumentMetadata};
use dms_domain::library::{FileNamePolicy, RelativePath};
use dms_domain::link::DocumentLink;
use dms_domain::taxonomy::CategoryId;
use dms_domain::text;

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, SiblingNames};
use crate::letter::{SaveLetterInput, SaveLetterOutcome, letter_document};
use crate::library::CatalogStore;
use crate::port::pdf::PdfEditor;
use crate::port::state::ActivityKind;
use crate::port::storage::FolderOperation;
use crate::port::system::{Clock, IdGenerator};
use crate::sync::Replicator;

/// The category a saved letter gets, if the library has one of this name (ignoring case).
const LETTER_CATEGORY: &str = "Letter";
/// The label of the links from a letter to its source documents.
const LINK_LABEL: &str = "letter";
/// Room for the date in front and ".pdf" behind the subject in a file name.
const SUBJECT_IN_NAME: usize = FileNamePolicy::MAX_LENGTH - 20;

/// Saves an edited letter as a PDF named `<date> <subject>.pdf` in the chosen folder of every library folder, then
/// catalogs it: title = subject, correspondent = recipient, document date = today, the category "Letter" if there
/// is one, linked to its source documents.
pub struct SaveLetter {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl SaveLetter {
    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, pdf, replicator, names, activity, clock, ids }
    }

    pub fn run(&self, input: &SaveLetterInput) -> SaveLetterOutcome {
        let Some(library) = self.store.current() else {
            return SaveLetterOutcome::Refused(OperationOutcome::LibraryNotOpen);
        };
        let subject = text::trim(&input.subject);
        if subject.is_empty() {
            let error = DomainError::InvalidArgument("The letter needs a subject".to_owned());
            return SaveLetterOutcome::Refused(OperationOutcome::InvalidInput(error));
        }
        if input.document_ids.iter().any(|id| !library.catalog.documents().contains_key(id)) {
            return SaveLetterOutcome::Refused(OperationOutcome::NotFound);
        }
        let today = self.clock.today();
        let path = match self.free_path(&library, &input.folder, &format!("{today} {subject}")) {
            Ok(path) => path,
            Err(outcome) => return SaveLetterOutcome::Refused(outcome),
        };
        let document =
            letter_document(input.sender.as_ref(), &input.recipient, today, subject, &input.body, input.language);
        let facts = match self.pdf.write_text_document(library.folders.primary(), &path, &document) {
            Ok(facts) => facts,
            Err(error) => return SaveLetterOutcome::Refused(OperationOutcome::StorageFailed(error)),
        };
        let pending = self.replicator.distribute(
            &library.folders,
            &library.folders.sync_folders(),
            &FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: false },
        );
        let id = DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank");
        let letter = Document {
            metadata: DocumentMetadata {
                title: Some(subject.to_owned()),
                document_date: Some(today),
                correspondent: Some(text::trim(&input.recipient.name).to_owned()).filter(|name| !name.is_empty()),
            },
            category_id: letter_category(&library.catalog),
            ..Document::new(
                id.clone(),
                path.clone(),
                facts.content_hash.expect("invariant: a written file is always hashed"),
                facts.size,
                self.clock.now(),
            )
            .expect("invariant: a letter file is never the root")
        };
        let outcome = self
            .store
            .commit(|catalog| add_linked(catalog, &letter, &input.document_ids))
            .with_pending_folders(&pending);
        let OperationOutcome::Done { pending_folders, .. } = outcome else {
            return SaveLetterOutcome::Refused(outcome);
        };
        self.activity.record(ActivityKind::DocumentAdded, &format!("Saved the letter {path}"));
        SaveLetterOutcome::Saved { document_id: id, path, pending_folders }
    }

    /// A free path for `name` (made valid) with ".pdf" in `folder`.
    #[allow(clippy::result_large_err)] // the refusal is the outcome of the whole action, built once
    fn free_path(
        &self,
        library: &LibrarySnapshot,
        folder: &RelativePath,
        name: &str,
    ) -> Result<RelativePath, OperationOutcome> {
        let name = format!("{}.pdf", FileNamePolicy::sanitize(text::take_utf16(name, SUBJECT_IN_NAME)));
        let siblings = self.names.of(library, folder).map_err(OperationOutcome::StorageFailed)?;
        let siblings: Vec<&str> = siblings.iter().map(String::as_str).collect();
        folder.resolve(&FileNamePolicy::next_free_name(&name, &siblings)).map_err(OperationOutcome::InvalidInput)
    }
}

fn letter_category(catalog: &Catalog) -> Option<CategoryId> {
    let wanted = text::lowercase(LETTER_CATEGORY);
    catalog
        .categories()
        .values()
        .find(|category| text::lowercase(&category.name) == wanted)
        .map(|category| category.id.clone())
}

fn add_linked(catalog: &Catalog, letter: &Document, sources: &[DocumentId]) -> Result<CatalogUpdate, DomainError> {
    let links = sources
        .iter()
        .map(|source| DocumentLink::new(letter.id.clone(), source.clone(), Some(LINK_LABEL)))
        .collect::<Result<Vec<_>, _>>()?;
    let added = catalog.add_document(letter.clone())?;
    Ok(links.into_iter().fold(added, |update, link| update.and_then(|catalog| catalog.link(link))))
}
