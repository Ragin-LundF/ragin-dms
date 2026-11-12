use std::collections::HashSet;
use std::sync::Arc;

use dms_domain::document::{Document, DocumentKind};
use dms_domain::health::{CopyObservation, CopyState, HealthFinding, HealthJudge};
use dms_domain::library::{LibraryFolder, LibraryFolderId};

use crate::common::{LibrarySnapshot, Progress};
use crate::health::{VerifyOutcome, VerifyProgress};
use crate::library::CatalogStore;
use crate::port::health::{FolderCheck, HealthReport, HealthStore};
use crate::port::pdf::PdfEditor;
use crate::port::storage::{DocumentStorage, StorageFailure, StorageResult};
use crate::port::system::Clock;

/// The report is saved after this many documents, so a crash loses little of a long pass.
const SAVE_EVERY: usize = 50;

/// Reads every copy of every document in every reachable folder and compares it with the catalog: size, SHA-256
/// and, for PDFs, whether it opens. Files a sync client keeps only in the cloud are skipped and counted, never
/// downloaded; a folder that is offline keeps its last findings and is checked next time. Changes no file and no
/// catalog: the findings go into this device's [`HealthReport`].
///
/// Cancellable between two documents and resumable: documents are checked in the order of their ids, and an
/// unfinished pass continues after the last document it checked.
pub struct VerifyLibrary {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    pdf: Arc<dyn PdfEditor>,
    health: Arc<dyn HealthStore>,
    clock: Arc<dyn Clock>,
}

impl VerifyLibrary {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        pdf: Arc<dyn PdfEditor>,
        health: Arc<dyn HealthStore>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { store, storage, pdf, health, clock }
    }

    pub fn run(&self, progress: &dyn Progress<VerifyProgress>) -> VerifyOutcome {
        let Some(library) = self.store.current() else {
            return VerifyOutcome::LibraryNotOpen;
        };
        let mut report = match self.health.load() {
            Ok(stored) => self.pass_of(stored, &library),
            Err(error) => return VerifyOutcome::StorageFailed(error),
        };
        let mut documents: Vec<&Document> = library.catalog.documents().values().collect();
        documents.sort_by(|first, second| first.id.value().cmp(second.id.value()));
        let total = documents.len();
        let resume_after = report.resume_after.as_ref().map(|id| id.value().to_owned());
        let start = resume_after.map_or(0, |after| documents.partition_point(|document| document.id.value() <= &after));
        let mut offline: HashSet<LibraryFolderId> =
            report.folders.iter().filter(|check| check.offline).map(|check| check.folder_id.clone()).collect();
        for (index, document) in documents.iter().enumerate().skip(start) {
            if progress.is_cancelled() {
                return self.saved(report, VerifyOutcome::Cancelled);
            }
            progress.report(&VerifyProgress { done: index, total, current: document.path.clone() });
            self.check(document, &library, &mut report, &mut offline);
            report.resume_after = Some(document.id.clone());
            if (index + 1) % SAVE_EVERY == 0 {
                // A failed save only loses the resume point; the pass goes on and saves at its end.
                let _ = self.health.save(&report);
            }
        }
        finish(&mut report, &library, self.clock.now());
        self.saved(report, VerifyOutcome::Finished)
    }

    /// The report to continue: the unfinished pass of this library, or a new pass that keeps the last findings
    /// until each document is checked again.
    fn pass_of(&self, stored: Option<HealthReport>, library: &LibrarySnapshot) -> HealthReport {
        let library_id = library.catalog.library_id();
        let mut report = stored
            .filter(|report| &report.library_id == library_id)
            .unwrap_or_else(|| HealthReport::new(library_id.clone()));
        if report.resume_after.is_none() {
            report.started_at = Some(self.clock.now());
            report.folders.clear();
        }
        for folder in library.folders.folders() {
            if !report.folders.iter().any(|check| check.folder_id == folder.id) {
                report.folders.push(FolderCheck::new(folder.id.clone()));
            }
        }
        report
    }

    fn check(
        &self,
        document: &Document,
        library: &LibrarySnapshot,
        report: &mut HealthReport,
        offline: &mut HashSet<LibraryFolderId>,
    ) {
        let mut copies = Vec::new();
        for folder in library.folders.folders() {
            if offline.contains(&folder.id) {
                continue;
            }
            if let Some(state) = self.observe(folder, document) {
                count(report, &folder.id, &state);
                copies.push(CopyObservation { folder_id: folder.id.clone(), state });
            } else {
                offline.insert(folder.id.clone());
                if let Some(check) = report.folders.iter_mut().find(|check| check.folder_id == folder.id) {
                    check.offline = true;
                }
            }
        }
        let findings = HealthJudge::judge(document, &copies, &library.folders.primary().id);
        // Findings in offline folders stay until the folder is checked again.
        report.findings.retain(|finding| {
            finding.document_id() != &document.id
                || matches!(finding, HealthFinding::Damaged { folder_id, .. } if offline.contains(folder_id))
        });
        for finding in findings {
            if !report.findings.contains(&finding) {
                report.findings.push(finding);
            }
        }
    }

    /// What `folder` holds of `document`; `None` when the folder is offline.
    fn observe(&self, folder: &LibraryFolder, document: &Document) -> Option<CopyState> {
        let facts = match self.storage.read_local_facts(folder, &document.path) {
            Ok(Some(facts)) => facts,
            Ok(None) => return Some(CopyState::NotDownloaded),
            Err(error) => {
                return match error.reason {
                    StorageFailure::FolderUnreachable => None,
                    StorageFailure::NotFound => Some(CopyState::Missing),
                    _ => Some(CopyState::ReadFailed),
                };
            }
        };
        let Some(content_hash) = facts.content_hash else {
            return Some(CopyState::ReadFailed);
        };
        let opens = (document.kind() == DocumentKind::Pdf).then(|| opens(self.pdf.page_count(folder, &document.path)));
        Some(CopyState::Read { content_hash, size: facts.size, opens })
    }

    fn saved(&self, report: HealthReport, outcome: fn(HealthReport) -> VerifyOutcome) -> VerifyOutcome {
        match self.health.save(&report) {
            Ok(()) => outcome(report),
            Err(error) => VerifyOutcome::StorageFailed(error),
        }
    }
}

fn opens(pages: StorageResult<usize>) -> bool {
    pages.is_ok_and(|pages| pages > 0)
}

fn count(report: &mut HealthReport, folder_id: &LibraryFolderId, state: &CopyState) {
    let Some(check) = report.folders.iter_mut().find(|check| &check.folder_id == folder_id) else { return };
    match state {
        CopyState::Missing => check.missing += 1,
        CopyState::NotDownloaded => check.not_downloaded += 1,
        CopyState::ReadFailed | CopyState::Read { .. } => check.checked += 1,
    }
}

/// Ends a complete pass: findings of documents and folders that are gone are dropped.
fn finish(report: &mut HealthReport, library: &LibrarySnapshot, now: chrono::DateTime<chrono::Utc>) {
    let documents = library.catalog.documents();
    report.findings.retain(|finding| {
        documents.contains_key(finding.document_id())
            && match finding {
                HealthFinding::Damaged { folder_id, source_folder_id, .. } => {
                    library.folders.find(folder_id).is_some() && library.folders.find(source_folder_id).is_some()
                }
                HealthFinding::Unreadable { .. } => true,
            }
    });
    report.folders.retain(|check| library.folders.find(&check.folder_id).is_some());
    report.resume_after = None;
    report.finished_at = Some(now);
}
