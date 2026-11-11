use std::sync::Arc;

use dms_domain::catalog::CatalogUpdate;
use dms_domain::library::{LibraryFolderId, RelativePath};
use dms_domain::restructure::{PlannedMove, RestructurePlan};
use dms_domain::text;
use indexmap::{IndexMap, IndexSet};

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, Progress};
use crate::library::CatalogStore;
use crate::port::plan::{PlanStore, RestructureRecord};
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::port::system::Clock;
use crate::restructure::RestructureOutcome;
use crate::sync::{Replication, Replicator};

const BATCH_SIZE: usize = 50;

/// Applies an approved restructure: moves the files one by one in the primary, journals each move for the sync
/// folders, and commits the catalog every 50 moves. Cancelling stops between two files and still commits what was
/// done, so the catalog always matches the disk. What was done is recorded in `.dms-plans/` (replicated), so it can
/// be reversed.
///
/// Files are moved before the catalog is committed. A batch whose commit fails has still moved: it is tried once
/// more at the end, and what is still not catalogued is recorded and reported on its own.
pub struct ApplyRestructurePlan {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    plans: Arc<dyn PlanStore>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
}

#[derive(Default)]
struct Run {
    batch: Vec<PlannedMove>,
    done: Vec<PlannedMove>,
    failed: Vec<PlannedMove>,
    not_catalogued: Vec<PlannedMove>,
    pending: IndexSet<LibraryFolderId>,
}

impl ApplyRestructurePlan {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        plans: Arc<dyn PlanStore>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { store, storage, replicator, plans, activity, clock }
    }

    /// Reports `(done, total)` after each move. A cancelled run returns its outcome with `cancelled: true`, where
    /// Kotlin committed the same and then rethrew the cancellation.
    pub fn run(
        &self,
        plan: &RestructurePlan,
        skipped: usize,
        progress: &dyn Progress<(usize, usize)>,
    ) -> RestructureOutcome {
        let Some(library) = self.store.current() else {
            return RestructureOutcome::LibraryNotOpen;
        };
        let mut run = Run::default();
        let mut cancelled = false;
        for (index, planned) in plan.moves.iter().enumerate() {
            if progress.is_cancelled() {
                cancelled = true;
                break;
            }
            self.move_file(&library, planned, &mut run);
            if run.batch.len() >= BATCH_SIZE {
                self.commit(&mut run);
            }
            progress.report(&(index + 1, plan.moves.len()));
        }
        self.finish(plan, run, skipped, cancelled)
    }

    fn finish(&self, plan: &RestructurePlan, mut run: Run, skipped: usize, cancelled: bool) -> RestructureOutcome {
        self.commit(&mut run);
        // One more try for the moves whose catalog commit failed; their files have moved already.
        let retry = std::mem::take(&mut run.not_catalogued);
        run.batch.extend(retry);
        self.commit(&mut run);
        let recorded: Vec<PlannedMove> = run.done.iter().chain(&run.not_catalogued).cloned().collect();
        let record = self.record(plan, recorded, &mut run);
        if !run.done.is_empty() {
            self.activity
                .record(ActivityKind::Restructured, &format!("{} documents by {}", run.done.len(), plan.template));
        }
        let sources: IndexSet<RelativePath> = run.done.iter().filter_map(|planned| planned.from.parent()).collect();
        RestructureOutcome::Applied {
            moved: run.done.len(),
            empty_folders: self.empty_folders(&sources),
            failed: run.failed,
            not_catalogued: run.not_catalogued,
            skipped,
            pending_folders: run.pending,
            record,
            cancelled,
        }
    }

    /// Moves one file in the primary and journals the move for the sync folders. A document the catalog already has
    /// at the target only needs its file to follow; this is how a reversal brings back a file whose move was never
    /// catalogued.
    fn move_file(&self, library: &LibrarySnapshot, planned: &PlannedMove, run: &mut Run) {
        let movable = self.store.with_catalog(|open| {
            let catalog = open.unwrap_or(&library.catalog);
            let current = catalog.documents().get(&planned.document_id).map(|document| &document.path);
            let owner_id = catalog.find_by_path(&planned.to).map_or(&planned.document_id, |owner| &owner.id);
            let at_either_end = current.is_some_and(|path| *path == planned.from || *path == planned.to);
            at_either_end && *owner_id == planned.document_id
        });
        if !movable {
            run.failed.push(planned.clone());
            return;
        }
        let replication = self.replicator.replicate(
            &library.folders,
            |primary| self.storage.move_entry(primary, &planned.from, &planned.to),
            &FolderOperation::Move { from: planned.from.clone(), to: planned.to.clone() },
        );
        match replication {
            Replication::PrimaryFailed(_) => run.failed.push(planned.clone()),
            Replication::Done { pending_folders, .. } => {
                run.batch.push(planned.clone());
                run.pending.extend(pending_folders);
            }
        }
    }

    /// Commits the moves of the current batch in one catalog revision.
    fn commit(&self, run: &mut Run) {
        if run.batch.is_empty() {
            return;
        }
        let batch = std::mem::take(&mut run.batch);
        let outcome = self.store.commit(|catalog| {
            let moves: IndexMap<_, _> = batch
                .iter()
                .filter(|planned| {
                    catalog.documents().get(&planned.document_id).is_some_and(|document| document.path == planned.from)
                })
                .map(|planned| (planned.document_id.clone(), planned.to.clone()))
                .collect();
            if moves.is_empty() { Ok(CatalogUpdate::Applied(catalog.clone())) } else { catalog.move_documents(&moves) }
        });
        if let OperationOutcome::Done { pending_folders, .. } = outcome {
            run.done.extend(batch);
            run.pending.extend(pending_folders);
        } else {
            run.not_catalogued.extend(batch);
        }
    }

    fn record(&self, plan: &RestructurePlan, moves: Vec<PlannedMove>, run: &mut Run) -> Option<String> {
        if moves.is_empty() {
            return None;
        }
        let library = self.store.current()?;
        let now = self.clock.now();
        // UTC to the second: two records in one second overwrite each other (parity-checklist open point 16).
        let record = RestructureRecord {
            name: now.format("%Y%m%d-%H%M%S").to_string(),
            applied_at: now,
            template: plan.template.clone(),
            moves,
        };
        let replication = self.replicator.replicate(
            &library.folders,
            |primary| self.plans.write(primary, &record),
            &FolderOperation::CopyPlan(record.name.clone()),
        );
        match replication {
            Replication::PrimaryFailed(_) => None,
            Replication::Done { pending_folders, .. } => {
                run.pending.extend(pending_folders);
                Some(record.name)
            }
        }
    }

    /// Source folders the catalog has no documents in any more and whose primary copy holds nothing visible.
    fn empty_folders(&self, sources: &IndexSet<RelativePath>) -> Vec<RelativePath> {
        let Some(library) = self.store.current() else {
            return Vec::new();
        };
        let mut empty: Vec<RelativePath> = sources
            .iter()
            .filter(|folder| !folder.is_root())
            .filter(|folder| {
                let catalogued = library.catalog.documents().values().any(|document| document.path.is_inside(folder));
                let listed = self.storage.list_names(library.folders.primary(), folder);
                !catalogued && listed.is_ok_and(|names| names.iter().all(|name| name.starts_with('.')))
            })
            .cloned()
            .collect();
        empty.sort_by(|first, second| text::compare(first.value(), second.value()));
        empty
    }
}
