use std::sync::Arc;

use chrono::NaiveDate;
use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogUpdate};
use dms_domain::change::FileFacts;
use dms_domain::document::{Document, DocumentId, DocumentMetadata, FileNameDates};
use dms_domain::library::{FileNamePolicy, LibraryFolderId, RelativePath};
use dms_domain::link::DocumentLink;
use dms_domain::text;
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, SiblingNames};
use crate::document::TrashDocument;
use crate::library::CatalogStore;
use crate::mail::{EmailSource, ImportEmailsOutcome, ImportedEmail, is_mail_file, mail_document, mail_file_name};
use crate::port::mail::{MailConverter, MailMessage, MailSource};
use crate::port::pdf::PdfEditor;
use crate::port::state::ActivityKind;
use crate::port::storage::FolderOperation;
use crate::port::system::{Clock, IdGenerator};
use crate::sync::Replicator;

const LINK_LABEL: &str = "attachment";

/// Turns email files into documents: the mail as a PDF named `<date> <subject>.pdf` (title = subject,
/// correspondent = sender's name or domain, document date = sending date) and each attachment as a file of its own,
/// linked to the mail as "attachment". The attachments then wait for the usual suggestions. Files first in every
/// library folder, the catalog second. A catalogued mail file goes to the trash afterwards unless `keep_original`.
pub struct ImportEmails {
    store: Arc<CatalogStore>,
    mail: Arc<dyn MailConverter>,
    pdf: Arc<dyn PdfEditor>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    trash_document: Arc<TrashDocument>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

/// A mail resolved to its file and the folder its documents go into.
struct Located {
    source: MailSource,
    folder: RelativePath,
    original: Option<DocumentId>,
}

/// The files written for one mail.
struct Written {
    mail: Document,
    attachments: Vec<Document>,
    pending: IndexSet<LibraryFolderId>,
}

impl ImportEmails {
    #[allow(clippy::too_many_arguments)] // the ports and collaborators of one use case, wired once
    pub fn new(
        store: Arc<CatalogStore>,
        mail: Arc<dyn MailConverter>,
        pdf: Arc<dyn PdfEditor>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        trash_document: Arc<TrashDocument>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, mail, pdf, replicator, names, trash_document, activity, clock, ids }
    }

    pub fn run(&self, sources: &[EmailSource], keep_original: bool) -> ImportEmailsOutcome {
        if self.store.current().is_none() {
            return ImportEmailsOutcome::LibraryNotOpen;
        }
        ImportEmailsOutcome::Imported(sources.iter().map(|source| self.import(source, keep_original)).collect())
    }

    fn import(&self, source: &EmailSource, keep_original: bool) -> ImportedEmail {
        let Some(library) = self.store.current() else {
            return ImportedEmail::failed(String::new(), OperationOutcome::LibraryNotOpen);
        };
        let located = match locate(&library.catalog, source) {
            Ok(located) => located,
            Err((name, outcome)) => return ImportedEmail::failed(name, outcome),
        };
        let name = match &located.source {
            MailSource::Library(path) => path.file_name().to_owned(),
            MailSource::External(file) => file.file_name().to_owned(),
        };
        let written = match self.write(&library, &located) {
            Ok(written) => written,
            Err(outcome) => return ImportedEmail::failed(name, outcome),
        };
        let outcome = self.store.commit(|catalog| add_linked(catalog, &written)).with_pending_folders(&written.pending);
        if !matches!(outcome, OperationOutcome::Done { .. }) {
            return ImportedEmail::failed(name, outcome);
        }
        self.activity.record(ActivityKind::DocumentAdded, &format!("Imported the email {}", written.mail.path));
        let outcome = match located.original.filter(|_| !keep_original) {
            // The mail is in the catalog already; a failed trash leaves the mail file as it was.
            Some(original) => match self.trash_document.run(&original, true) {
                OperationOutcome::Done { catalog, pending_folders } => {
                    OperationOutcome::Done { catalog, pending_folders }.with_pending_folders(&written.pending)
                }
                _ => outcome,
            },
            None => outcome,
        };
        ImportedEmail {
            source: name,
            outcome,
            mail: Some(written.mail.id.clone()),
            attachments: written.attachments.iter().map(|document| document.id.clone()).collect(),
        }
    }

    /// Writes the mail's PDF and its attachments into the primary and the sync folders.
    #[allow(clippy::result_large_err)] // the refusal is the outcome of the whole mail, built once
    fn write(&self, library: &LibrarySnapshot, located: &Located) -> Result<Written, OperationOutcome> {
        let primary = library.folders.primary();
        let message = self.mail.read(primary, &located.source).map_err(OperationOutcome::StorageFailed)?;
        let date = message.sent_at.map_or_else(|| self.clock.today(), |sent_at| sent_at.date_naive());
        let mut taken: Vec<String> =
            self.names.of(library, &located.folder).map_err(OperationOutcome::StorageFailed)?.into_iter().collect();
        let mut pending = IndexSet::new();
        let pdf_path = free_path(&located.folder, &mail_file_name(date, &message), &mut taken)?;
        let facts = self
            .pdf
            .write_text_document(primary, &pdf_path, &mail_document(&message))
            .map_err(OperationOutcome::StorageFailed)?;
        pending.extend(self.distribute(library, &pdf_path));
        let mail = self.document(&pdf_path, &facts, mail_metadata(&message, date));
        let mut attachments = Vec::new();
        for (index, attachment) in message.attachments.iter().enumerate() {
            let path = free_path(&located.folder, &FileNamePolicy::sanitize(&attachment.name), &mut taken)?;
            let facts = self
                .mail
                .write_attachment(primary, &located.source, index, &path)
                .map_err(OperationOutcome::StorageFailed)?;
            pending.extend(self.distribute(library, &path));
            let document_date = FileNameDates::date_in(path.file_name()).or(Some(date));
            attachments.push(self.document(&path, &facts, DocumentMetadata { document_date, ..Default::default() }));
        }
        Ok(Written { mail, attachments, pending })
    }

    fn distribute(&self, library: &LibrarySnapshot, path: &RelativePath) -> IndexSet<LibraryFolderId> {
        self.replicator.distribute(
            &library.folders,
            &library.folders.sync_folders(),
            &FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: false },
        )
    }

    fn document(&self, path: &RelativePath, facts: &FileFacts, metadata: DocumentMetadata) -> Document {
        Document {
            metadata,
            ..Document::new(
                DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
                path.clone(),
                facts.content_hash.clone().expect("invariant: a written file is always hashed"),
                facts.size,
                self.clock.now(),
            )
            .expect("invariant: a written file is never the root")
        }
    }
}

#[allow(clippy::result_large_err)] // the refusal is the outcome of the whole mail, built once
fn locate(catalog: &Catalog, source: &EmailSource) -> Result<Located, (String, OperationOutcome)> {
    match source {
        EmailSource::Document(id) => {
            let Some(document) = catalog.documents().get(id) else {
                return Err((id.value().to_owned(), OperationOutcome::NotFound));
            };
            let name = document.path.file_name().to_owned();
            if !is_mail_file(&name) {
                let error = DomainError::InvalidArgument("Only .eml files can be imported as emails".to_owned());
                return Err((name, OperationOutcome::InvalidInput(error)));
            }
            Ok(Located {
                source: MailSource::Library(document.path.clone()),
                folder: document.path.parent().unwrap_or_else(RelativePath::root),
                original: Some(id.clone()),
            })
        }
        EmailSource::External { file, folder } => {
            if !is_mail_file(file.file_name()) {
                let error = DomainError::InvalidArgument("Only .eml files can be imported as emails".to_owned());
                return Err((file.file_name().to_owned(), OperationOutcome::InvalidInput(error)));
            }
            Ok(Located { source: MailSource::External(file.clone()), folder: folder.clone(), original: None })
        }
    }
}

/// A free path for `name` in `folder`, also free of the names this mail took already, which it then takes.
#[allow(clippy::result_large_err)] // the refusal is the outcome of the whole mail, built once
fn free_path(folder: &RelativePath, name: &str, taken: &mut Vec<String>) -> Result<RelativePath, OperationOutcome> {
    let free = FileNamePolicy::next_free_name(name, taken);
    let path = folder.resolve(&free).map_err(OperationOutcome::InvalidInput)?;
    taken.push(free);
    Ok(path)
}

fn mail_metadata(message: &MailMessage, date: NaiveDate) -> DocumentMetadata {
    let correspondent =
        message.from_name.as_deref().map(text::trim).filter(|name| !name.is_empty()).map(str::to_owned).or_else(|| {
            message.from_address.as_deref().and_then(|address| address.rsplit_once('@')).map(|it| it.1.to_owned())
        });
    let title = message.subject.as_deref().map(text::trim).filter(|it| !it.is_empty()).map(str::to_owned);
    DocumentMetadata { title, document_date: Some(date), correspondent }
}

fn add_linked(catalog: &Catalog, written: &Written) -> Result<CatalogUpdate, DomainError> {
    let links = written
        .attachments
        .iter()
        .map(|attachment| DocumentLink::new(written.mail.id.clone(), attachment.id.clone(), Some(LINK_LABEL)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut added = catalog.add_document(written.mail.clone())?;
    for attachment in &written.attachments {
        added = match added {
            CatalogUpdate::Applied(catalog) => catalog.add_document(attachment.clone())?,
            rejected @ CatalogUpdate::Rejected(_) => return Ok(rejected),
        };
    }
    Ok(links.into_iter().fold(added, |update, link| update.and_then(|catalog| catalog.link(link))))
}
