use std::sync::Arc;

use dms_domain::bill::BillPlanner;
use dms_domain::cases::CasePlanner;
use dms_domain::change::DetectedChange;
use dms_domain::contract::ContractPlanner;
use dms_domain::deadline::DeadlinePlanner;
use dms_domain::document::{ContentHash, DocumentId};
use dms_domain::emergency::EmergencyPlanner;
use dms_domain::expectation::ExpectationPlanner;
use dms_domain::field::FieldRole;
use dms_domain::health::HealthFinding;
use dms_domain::library::LibraryFolderId;
use dms_domain::restructure::FolderShape;
use dms_domain::rule::RuleLearner;
use dms_domain::task::OpenTask;
use indexmap::IndexMap;

use crate::common::LibrarySnapshot;
use crate::library::{CatalogStore, Dashboard, DashboardOutcome, FolderState, ScanResult};
use crate::port::health::HealthStore;
use crate::port::state::{ActivityLog, TaskStore};
use crate::port::storage::OperationJournal;
use crate::port::system::Clock;

const RECENT_ACTIVITY: usize = 20;
const UPCOMING: usize = 10;

/// Collects everything that waits for the user: pending suggestions, the changes of the last scan, catalogued
/// documents that are copies of each other, decisions of the last catalog merge, failed operations, flat folders
/// the restructure assistant could sort, deadlines within their lead time and ended retentions, and rules learned
/// from approved documents, the tasks of bills, expected documents, contracts, cases and the emergency folder, and
/// the findings of the last archive check —
/// minus the tasks the user dismissed, so a rejected rule proposal is not proposed
/// again. New files in the inbox are no tasks: the inbox lists them.
///
/// Local state that cannot be read (journal, task store, activity log) counts as empty, so the dashboard still
/// shows everything else (Kotlin threw).
pub struct BuildDashboard {
    store: Arc<CatalogStore>,
    tasks: Arc<dyn TaskStore>,
    journal: Arc<dyn OperationJournal>,
    activity: Arc<dyn ActivityLog>,
    clock: Arc<dyn Clock>,
    health: Arc<dyn HealthStore>,
}

impl BuildDashboard {
    pub fn new(
        store: Arc<CatalogStore>,
        tasks: Arc<dyn TaskStore>,
        journal: Arc<dyn OperationJournal>,
        activity: Arc<dyn ActivityLog>,
        clock: Arc<dyn Clock>,
        health: Arc<dyn HealthStore>,
    ) -> Self {
        Self { store, tasks, journal, activity, clock, health }
    }

    /// `scan` is the last scan, if there was one.
    pub fn run(&self, scan: Option<&ScanResult>) -> DashboardOutcome {
        let Some(library) = self.store.current() else {
            return DashboardOutcome::LibraryNotOpen;
        };
        let mut pending: IndexMap<LibraryFolderId, usize> = IndexMap::new();
        for operation in self.journal.pending().unwrap_or_default() {
            *pending.entry(operation.folder_id).or_default() += 1;
        }
        let open_tasks = self.open_tasks(&library, scan, &pending);
        let dismissed = self.tasks.dismissed_keys().unwrap_or_default();
        let documents = library.catalog.documents();
        let dashboard = Dashboard {
            tasks: open_tasks.into_iter().filter(|task| !dismissed.contains(&task.key())).collect(),
            folders: library
                .folders
                .folders()
                .iter()
                .map(|folder| FolderState {
                    folder: folder.clone(),
                    reachable: scan.map(|scan| scan.scanned_folders.contains(&folder.id)),
                    pending_operations: pending.get(&folder.id).copied().unwrap_or(0),
                })
                .collect(),
            document_count: documents.len(),
            uncategorized_count: documents.values().filter(|document| document.category_id.is_none()).count(),
            recent_activity: self.activity.recent(RECENT_ACTIVITY).unwrap_or_default(),
            upcoming: DeadlinePlanner::upcoming(&library.catalog, self.clock.today(), UPCOMING),
        };
        DashboardOutcome::Built(dashboard)
    }

    fn open_tasks(
        &self,
        library: &LibrarySnapshot,
        scan: Option<&ScanResult>,
        pending: &IndexMap<LibraryFolderId, usize>,
    ) -> Vec<OpenTask> {
        let catalog = &library.catalog;
        let documents = catalog.documents();
        let inbox = &catalog.settings().layout.inbox;
        let mut tasks: Vec<OpenTask> = self
            .tasks
            .suggestions()
            .unwrap_or_default()
            .into_iter()
            .filter(|suggestion| documents.contains_key(&suggestion.document_id))
            .map(OpenTask::ReviewSuggestion)
            .collect();
        if let Some(scan) = scan {
            tasks.extend(
                scan.changes
                    .iter()
                    .filter(|change| !matches!(change, DetectedChange::NewFile(new) if inbox.contains(&new.path)))
                    .map(|change| OpenTask::for_change(change.clone())),
            );
        }
        let mut copies: IndexMap<(&ContentHash, u64), Vec<DocumentId>> = IndexMap::new();
        for document in documents.values() {
            copies.entry((&document.content_hash, document.size)).or_default().push(document.id.clone());
        }
        tasks.extend(copies.into_iter().filter(|(_, ids)| ids.len() > 1).map(|((content_hash, _), document_ids)| {
            OpenTask::ResolveDuplicates { content_hash: content_hash.clone(), document_ids }
        }));
        tasks.extend(self.tasks.merge_conflicts().unwrap_or_default().into_iter().map(OpenTask::ReviewCatalogMerge));
        tasks.extend(
            pending.iter().map(|(folder_id, count)| OpenTask::RetryFailedOperations {
                folder_id: folder_id.clone(),
                count: *count,
            }),
        );
        tasks.extend(
            FolderShape::analyse(catalog, inbox, FolderShape::DEFAULT_THRESHOLD)
                .into_iter()
                .map(|flat| OpenTask::RestructureSuggested { folder: flat.path, file_count: flat.file_count }),
        );
        tasks.extend(DeadlinePlanner::due_tasks(catalog, self.clock.today(), FieldRole::default_lead_days));
        tasks.extend(RuleLearner::proposals(catalog).into_iter().map(OpenTask::ReviewLearnedRule));
        tasks.extend(self.feature_tasks(catalog));
        tasks.extend(self.health_tasks(library));
        tasks
    }

    /// The findings of this device's last archive check whose document and folders still exist.
    fn health_tasks(&self, library: &LibrarySnapshot) -> Vec<OpenTask> {
        let Ok(Some(report)) = self.health.load() else { return Vec::new() };
        if &report.library_id != library.catalog.library_id() {
            return Vec::new();
        }
        let documents = library.catalog.documents();
        let folder_known = |id: &LibraryFolderId| library.folders.find(id).is_some();
        report
            .findings
            .into_iter()
            .filter(|finding| documents.contains_key(finding.document_id()))
            .filter_map(|finding| match finding {
                HealthFinding::Damaged { document_id, folder_id, source_folder_id } => (folder_known(&folder_id)
                    && folder_known(&source_folder_id))
                .then_some(OpenTask::DamagedCopy { document_id, folder_id, source_folder_id }),
                HealthFinding::Unreadable { document_id } => Some(OpenTask::UnreadableDocument { document_id }),
            })
            .collect()
    }

    /// The tasks of bills, expected documents, contracts, cases and the emergency folder.
    fn feature_tasks(&self, catalog: &dms_domain::catalog::Catalog) -> Vec<OpenTask> {
        let today = self.clock.today();
        let mut tasks = BillPlanner::due_tasks(catalog, today);
        let snoozes = self.tasks.snoozes().unwrap_or_default();
        tasks.extend(ExpectationPlanner::due_tasks(catalog, today, &snoozes));
        tasks.extend(ContractPlanner::due_tasks(catalog, today));
        tasks.extend(CasePlanner::due_tasks(catalog, today));
        tasks.extend(EmergencyPlanner::due_tasks(catalog, today));
        tasks
    }
}
